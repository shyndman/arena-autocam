use num_traits::cast::ToPrimitive;
use stepper::{
    drivers::a4988::A4988, fugit::NanosDurationU32 as Nanoseconds, motion_control,
    ramp_maker, Direction, Stepper,
};

struct Pin(&'static str);
impl embedded_hal::digital::ErrorType for Pin {
    type Error = core::convert::Infallible;
}
impl stepper::embedded_hal::digital::OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        eprintln!("{}: 0", self.0);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        eprintln!("{}: 1", self.0);
        Ok(())
    }
}

struct Timer<const TIMER_HZ: u32> {
    duration: fugit::TimerDurationU32<TIMER_HZ>,
}
impl<const TIMER_HZ: u32> Timer<TIMER_HZ> {
    pub fn new() -> Self {
        Self {
            duration: fugit::TimerDurationU32::from_ticks(0),
        }
    }
}
impl<const TIMER_HZ: u32> fugit_timer::Timer<TIMER_HZ> for Timer<TIMER_HZ> {
    type Error = std::convert::Infallible;
    fn now(&mut self) -> fugit::TimerInstantU32<TIMER_HZ> {
        todo!()
    }
    fn start(&mut self, val: fugit::TimerDurationU32<TIMER_HZ>) -> Result<(), Self::Error> {
        self.duration = val;
        Ok(())
    }
    fn cancel(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        if self.duration.ticks() != 0 {
            delay_ns(Nanoseconds::from_ticks(self.duration.to_nanos()));
        }

        Ok(())
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
        Ok(fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(
            Num::to_u32(&delay).expect("the delay to convert"),
        ))
    }
}

fn delay_ns(ns: Nanoseconds) {
    let t = ns.to_nanos();
    eprintln!("Delaying for {}ns", t);

    let mut i = 0;
    unsafe {
        // Volatile writes are slow enough to count down the time
        // until the next tick
        while std::ptr::read_volatile(&mut i) < t {
            std::ptr::write_volatile(&mut i, std::ptr::read_volatile(&mut i) + 1);
        }
    }
}

fn main() -> Result<
    (),
    stepper::Error<
        core::convert::Infallible,
        core::convert::Infallible,
        core::convert::Infallible,
        core::convert::Infallible,
    >,
> {
    let step = Pin("step");
    let dir = Pin("dir");
    let mut timer = Timer::<1_000_000>::new();
    let target_accel = Num::from_num(0.003); // steps / tick^2; 1000 steps / s^2
    let max_speed = Num::from_num(0.001); // steps / tick; 1000 steps / s
    let profile = ramp_maker::Trapezoidal::new(target_accel);

    let mut stepper = Stepper::from_driver(A4988::new())
        // Enable direction control
        .enable_direction_control(dir, Direction::Forward, &mut timer)?
        // Enable step control
        .enable_step_control(step)
        // Enable motion control using the software fallback
        .enable_motion_control((timer, profile, DelayToTicks));

    let target_step = 2000;
    stepper.move_to_position(max_speed, target_step).wait()?;

    Ok(())
}
