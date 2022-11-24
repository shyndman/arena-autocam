use std::task::Poll;

use anyhow::Result;
use embedded_hal::digital::ErrorType;
use fugit::{NanosDurationU32 as Nanoseconds, TimerDurationU32 as TimerDuration};
use fugit_timer::Timer as TimerTrait;
use stepper::{
    motion_control::{self, DelayToTicks, TimeConversionError},
    traits::{SetDirection, Step},
    Direction, SetDirectionFuture, StepFuture,
};

pub enum State<Driver, Timer, Delay, const TIMER_HZ: u32> {
    Idle {
        driver: Driver,
        timer: Timer,
    },
    SetDirection(SetDirectionFuture<Driver, Timer, TIMER_HZ>),
    Step {
        future: StepFuture<Driver, Timer, TIMER_HZ>,
        delay: Delay,
    },
    StepDelay {
        driver: Driver,
        timer: Timer,
    },
    Invalid,
}

pub fn update<Driver, Timer, Convert, Delay, const TIMER_HZ: u32>(
    mut state: State<Driver, Timer, Delay, TIMER_HZ>,
    current_step: &mut i32,
    current_target_step: &mut i32,
    current_direction: &mut Direction,
    convert: &Convert,
) -> (
    Result<
        bool,
        motion_control::Error<
            <Driver as SetDirection>::Error,
            <<Driver as SetDirection>::Dir as ErrorType>::Error,
            <Driver as Step>::Error,
            <<Driver as Step>::Step as ErrorType>::Error,
            Timer::Error,
            Convert::Error,
        >,
    >,
    State<Driver, Timer, Delay, TIMER_HZ>,
)
where
    Driver: SetDirection + Step,
    Timer: TimerTrait<TIMER_HZ>,
    Convert: DelayToTicks<Delay, TIMER_HZ>,
{
    loop {
        match state {
            State::Idle { driver, timer } => {
                // Being idle can mean that there's actually nothing to do, or
                // it might just be a short breather before more work comes in.

                // if let Some(direction) = new_motion.take() {
                //     // A new motion has been started. This might override an
                //     // ongoing one, but it makes no difference here.
                //     //
                //     // Let's update the state, but don't return just yet. We
                //     // have more stuff to do (polling the future).
                //     state = State::SetDirection(SetDirectionFuture::new(
                //         direction, driver, timer,
                //     ));
                //     *current_direction = direction;
                //     continue;
                // }

                // No new motion has been started, but we might still have an
                // ongoing one. Let's ask the motion profile.
                // if let Some(delay) = profile.next_delay() {
                //     // There's a motion ongoing. Let's start the next step, but
                //     // again, don't return yet. The future needs to be polled.
                //     state = State::Step {
                //         future: StepFuture::new(driver, timer),
                //         delay,
                //     };
                //     continue;
                // }

                // Now we know that there's truly nothing to do. Return to the
                // caller and stay idle.
                return (Ok(false), State::Idle { driver, timer });
            }
            State::SetDirection(mut future) => {
                match future.poll() {
                    Poll::Ready(Ok(())) => {
                        // Direction has been set. Set state back to idle, so we
                        // can figure out what to do next in the next loop
                        // iteration.
                        let (driver, timer) = future.release();
                        state = State::Idle { driver, timer };
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        // Error happened while setting direction. We need to
                        // let the caller know.
                        //
                        // The state stays as it is. For all we know, the error
                        // can be recovered from.
                        // anyhow::Error::msg("set_direction")
                        return (
                            Err(motion_control::Error::SetDirection(err)),
                            State::SetDirection(future),
                        );
                    }
                    Poll::Pending => {
                        // Still busy setting direction. Let caller know.
                        return (Ok(true), State::SetDirection(future));
                    }
                }
            }
            State::Step { mut future, delay } => {
                match future.poll() {
                    Poll::Ready(Ok(())) => {
                        // A step was made. Now we need to wait out the rest of
                        // the step delay before we can do something else.

                        *current_step += *current_direction as i32;

                        let (driver, mut timer) = future.release();
                        let delay_left: TimerDuration<TIMER_HZ> =
                            match delay_left(delay, Driver::PULSE_LENGTH, convert) {
                                Ok(delay_left) => delay_left,
                                Err(err) => {
                                    return (
                                        Err(motion_control::Error::TimeConversion(err)),
                                        State::Idle { driver, timer },
                                    )
                                }
                            };

                        if let Err(err) = timer.start(delay_left) {
                            return (
                                Err(motion_control::Error::StepDelay(err)),
                                State::Idle { driver, timer },
                            );
                        }

                        state = State::StepDelay { driver, timer };
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        // Error happened while stepping. Need to
                        // let the caller know.
                        //
                        // State stays as it is. For all we know,
                        // the error can be recovered from.
                        return (
                            Err(motion_control::Error::Step(err)),
                            State::Step { future, delay },
                        );
                    }
                    Poll::Pending => {
                        // Still stepping. Let caller know.
                        return (Ok(true), State::Step { future, delay });
                    }
                }
            }
            State::StepDelay { driver, mut timer } => {
                match timer.wait() {
                    Ok(()) => {
                        // We've waited out the step delay. Return to idle
                        // state, to figure out what's next.
                        state = State::Idle { driver, timer };
                        continue;
                    }
                    Err(nb::Error::WouldBlock) => {
                        // The timer is still running. Let the user know.
                        return (Ok(true), State::StepDelay { driver, timer });
                    }
                    Err(nb::Error::Other(err)) => {
                        // Error while trying to wait. Need to tell the caller.
                        return (
                            Err(motion_control::Error::StepDelay(err)),
                            State::StepDelay { driver, timer },
                        );
                    }
                }
            }
            State::Invalid => {
                // This can only happen if this closure panics, the
                // user catches the panic, then attempts to
                // continue.
                //
                // A panic in this closure is always going to be a
                // bug, and once that happened, we're in an invalid
                // state. Not a lot we can do about it.
                panic!("Invalid internal state, caused by a previous panic.")
            }
        }
    }
}

fn delay_left<Delay, Convert, const TIMER_HZ: u32>(
    delay: Delay,
    pulse_length: Nanoseconds,
    convert: &Convert,
) -> Result<TimerDuration<TIMER_HZ>, TimeConversionError<Convert::Error>>
where
    Convert: DelayToTicks<Delay, TIMER_HZ>,
{
    let delay: TimerDuration<TIMER_HZ> = convert
        .delay_to_ticks(delay)
        .map_err(|err| TimeConversionError::DelayToTicks(err))?;
    let pulse_length: TimerDuration<TIMER_HZ> = pulse_length.convert();

    let delay_left = delay - pulse_length;
    Ok(delay_left)
}
