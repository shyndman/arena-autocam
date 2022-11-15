use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use fugit::RateExtU32;
use log::{debug, info};
use num_traits::cast::ToPrimitive;
use rppal::gpio::Gpio;
use stepper::{drivers::a4988::A4988, motion_control, ramp_maker, Direction, Stepper};

struct Timer<const TIMER_HZ: u32> {
    running: bool,
    timer_start: Instant,
    start_ts: Instant,
    duration: fugit::TimerDurationU32<TIMER_HZ>,
    tick_duration: fugit::TimerDurationU32<TIMER_HZ>,
}

impl<const TIMER_HZ: u32> Timer<TIMER_HZ> {
    pub fn new() -> Self {
        Self {
            timer_start: Instant::now(),
            running: true,
            start_ts: Instant::now(),
            duration: fugit::TimerDurationU32::from_ticks(0),
            tick_duration: TIMER_HZ.Hz::<TIMER_HZ, 1>().into_duration(),
        }
    }

    pub fn std_elapsed(&mut self) -> Duration {
        self.start_ts.elapsed()
    }
    pub fn std_duration(&mut self) -> Duration {
        Duration::from_nanos(self.duration.to_nanos() as u64)
    }
}
impl<const TIMER_HZ: u32> fugit_timer::Timer<TIMER_HZ> for Timer<TIMER_HZ> {
    type Error = anyhow::Error;

    fn now(&mut self) -> fugit::TimerInstantU32<TIMER_HZ> {
        let ticks =
            self.timer_start.elapsed().as_micros() / self.tick_duration.to_micros() as u128;
        fugit::TimerInstantU32::from_ticks(ticks as u32)
    }

    fn start(&mut self, val: fugit::TimerDurationU32<TIMER_HZ>) -> Result<(), Self::Error> {
        if !self.running {
            return Err(anyhow!("Timer was already canceled"));
        }

        if self.duration.ticks() > 0 &&
            (self.std_elapsed().as_nanos() as u32) < self.duration.to_nanos()
        {
            return Err(anyhow!("start() called but duration has not yet expired"));
        }

        self.start_ts = Instant::now();
        self.duration = val;
        Ok(())
    }

    /// Tries to stop this timer.
    /// An error will be returned if the timer has already been canceled or was never started.
    /// An error is also returned if the timer is not `Periodic` and has already expired.
    fn cancel(&mut self) -> Result<(), Self::Error> {
        if self.running {
            self.running = false;
            Ok(())
        } else {
            Err(anyhow!("Timer was already canceled"))
        }
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        if !self.running {
            return Err(nb::Error::Other(anyhow!("Timer was already canceled")));
        }

        let duration = self.std_duration();
        loop {
            let elapsed = self.std_elapsed();
            if elapsed < duration {
                let remaining = duration - elapsed;
                debug!("sleeping for {}us", remaining.as_micros());
                delay_us(&duration);
                // if remaining > Duration::from_micros(60) {
                //     spin_sleep::SpinSleeper::default()
                //         .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread)
                //         .sleep(duration / 2);
                // } else {
                // }
            } else {
                break;
            }
        }

        let done_elapsed = self.std_elapsed();
        info!(
            "done in {}us, intended duration {}us ({:+}us)",
            done_elapsed.as_micros(),
            duration.as_micros(),
            if done_elapsed > duration {
                (done_elapsed - duration).as_micros() as i64
            } else {
                -((duration - done_elapsed).as_micros() as i64)
            }
        );

        Ok(())
    }
}

#[inline]
fn delay_us(duration: &Duration) {
    let us = duration.as_micros();
    let mut i = 0;
    unsafe {
        // Volatile writes are slow enough to count down the time
        // until the next tick
        while std::ptr::read_volatile(&mut i) < us {
            std::ptr::write_volatile(&mut i, std::ptr::read_volatile(&mut i) + 1);
        }
    }
}

type Num = fixed::FixedI64<typenum::U32>;

pub struct DelayToTicks;
impl<const TIMER_HZ: u32> motion_control::DelayToTicks<Num, TIMER_HZ> for DelayToTicks {
    type Error = core::convert::Infallible;
    fn delay_to_ticks(
        &self,
        delay: Num,
    ) -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error> {
        let ticks = fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(
            Num::to_u32(&delay).expect("the delay to convert"),
        );
        Ok(ticks)
    }
}

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    let gpio = Gpio::new()?;
    let step_pin = gpio.get(16)?.into_output();
    let direction_pin = gpio.get(21)?.into_output();

    let mut timer = Timer::<100_000>::new();
    let target_accel = Num::from_num(0.003); // steps / tick^2; 1000 steps / s^2
    let max_speed = Num::from_num(0.00001); // steps / tick; 1000 steps / s
    let profile = ramp_maker::Trapezoidal::new(target_accel);

    let mut stepper = Stepper::from_driver(A4988::new())
        // Enable direction control
        .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
        .map_err(|e| anyhow!("{:?}", e))?
        // Enable step control
        .enable_step_control(step_pin)
        // Enable motion control using the software fallback
        .enable_motion_control((timer, profile, DelayToTicks));

    let target_step = 800;
    stepper
        .move_to_position(max_speed, target_step)
        .wait()
        .map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}
