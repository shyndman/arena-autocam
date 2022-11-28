use std::{fmt::Debug, task::Poll};

use anyhow::{anyhow, Result};
use fugit::TimerDurationU32;
use fugit_timer::Timer as TimerTrait;
use num_rational::Rational32;
use stepper::{
    traits::{SetDirection, SetSleepMode, SetStepMode, Step},
    Direction, SetDirectionFuture, SetStepModeFuture, StepFuture,
};

pub(super) enum State<Driver, Timer, const TIMER_HZ: u32>
where
    Driver: SetStepMode,
{
    Idle {
        driver: Driver,
        timer: Timer,
    },
    SetDirection {
        future: SetDirectionFuture<Driver, Timer, TIMER_HZ>,
    },
    SetStepMode {
        future: SetStepModeFuture<Driver, Timer, TIMER_HZ>,
    },
    /// Delays by `delay` of time, then invokes the [State::Step] stage
    StepDelay {
        driver: Driver,
        /// This timer should already be configured with the desired delay
        timer: Timer,
    },
    Step {
        future: StepFuture<Driver, Timer, TIMER_HZ>,
    },
    Invalid,
}

/// Indicates whether a call to [`update()`] completed all of its work
/// [`UpdateProgress::Complete`], or still requires additional calls.
#[derive(PartialEq)]
pub enum FsmStatus {
    Ready,
    Pending,
}

/// Updates the velocity controller's state in place.
///
/// This function should be called repeatedly as long as `Ok(UpdateProgress::Pending)` is
/// being retuned.
pub(super) fn update<Driver, Timer, TimerError, const TIMER_HZ: u32>(
    mut state: State<Driver, Timer, TIMER_HZ>,
    next_velocity: &mut Option<f64>,
    next_delay: &mut Option<TimerDurationU32<TIMER_HZ>>,
    next_direction: &mut Option<Direction>,
    next_step_mode: &mut Option<Driver::StepMode>,
    current_velocity: &mut f64,
    current_direction: &mut Direction,
    current_step: &mut Rational32,
    current_step_mode: &mut Driver::StepMode,
) -> (Result<FsmStatus>, State<Driver, Timer, TIMER_HZ>)
where
    Driver: SetDirection + SetSleepMode + SetSleepMode + SetStepMode + Step,
    Timer: TimerTrait<TIMER_HZ, Error = TimerError>,
    TimerError: Debug,
{
    loop {
        match state {
            State::Idle { driver, mut timer } => {
                if let Some(velocity) = next_velocity.take() {
                    *current_velocity = velocity;
                }

                if let Some(direction) = next_direction.take() {
                    state = State::SetDirection {
                        future: SetDirectionFuture::new(direction, driver, timer),
                    };
                    *current_direction = direction;
                    continue;
                }

                if let Some(step_mode) = next_step_mode.take() {
                    state = State::SetStepMode {
                        future: SetStepModeFuture::new(step_mode, driver, timer),
                    };
                    *current_step_mode = step_mode;
                    continue;
                }

                if let Some(delay) = next_delay.take() {
                    if let Err(err) = timer.start(delay.to_owned()) {
                        return (Err(anyhow!("{:?}", err)), State::Idle { driver, timer });
                    }
                    state = State::StepDelay {
                        driver: driver,
                        timer: timer,
                    };
                    continue;
                }

                // Now we know that there's truly nothing to do. Return to the
                // caller and stay idle.
                return (Ok(FsmStatus::Ready), State::Idle { driver, timer });
            }
            State::SetDirection { mut future } => match future.poll() {
                Poll::Ready(Ok(())) => {
                    let (driver, timer) = future.release();
                    state = State::Idle { driver, timer };
                    continue;
                }
                Poll::Ready(Err(err)) => {
                    return (Err(anyhow!("{:?}", err)), State::SetDirection { future });
                }
                Poll::Pending => {
                    return (Ok(FsmStatus::Pending), State::SetDirection { future });
                }
            },
            State::SetStepMode { mut future } => match future.poll() {
                Poll::Ready(Ok(())) => {
                    let (driver, timer) = future.release();
                    state = State::Idle { driver, timer };
                    continue;
                }
                Poll::Ready(Err(err)) => {
                    return (Err(anyhow!("{:?}", err)), State::SetStepMode { future });
                }
                Poll::Pending => {
                    return (Ok(FsmStatus::Pending), State::SetStepMode { future });
                }
            },
            State::StepDelay { driver, mut timer } => {
                match timer.wait() {
                    Ok(()) => {
                        state = State::Step {
                            future: StepFuture::new(driver, timer),
                        };
                        continue;
                    }
                    Err(nb::Error::WouldBlock) => {
                        // The timer is still running. Let the user know.
                        return (Ok(FsmStatus::Pending), State::StepDelay { driver, timer });
                    }
                    Err(nb::Error::Other(err)) => {
                        // Error while trying to wait. Need to tell the caller.
                        return (
                            Err(anyhow!("{:?}", err)),
                            State::StepDelay { driver, timer },
                        );
                    }
                }
            }
            State::Step { mut future } => {
                match future.poll() {
                    Poll::Ready(Ok(())) => {
                        // A step was made. Now we need to wait out the rest of
                        // the step delay before we can do something else.
                        let step_base: u16 = current_step_mode.to_owned().into();
                        *current_step +=
                            Rational32::new(*current_direction as i32, step_base as i32);

                        let (driver, timer) = future.release();
                        state = State::Idle { driver, timer };
                        continue;
                    }
                    Poll::Ready(Err(err)) => {
                        // Error happened while stepping. Need to
                        // let the caller know.
                        //
                        // State stays as it is. For all we know,
                        // the error can be recovered from.
                        return (Err(anyhow!("{:?}", err)), State::Step { future });
                    }
                    Poll::Pending => {
                        // Still stepping. Let caller know.
                        return (Ok(FsmStatus::Pending), State::Step { future });
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
