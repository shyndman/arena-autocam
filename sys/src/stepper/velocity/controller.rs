#[allow(unused)]
use aa_foundation::prelude::*;
use anyhow::Result;
use fugit::{MillisDurationU32, TimerDurationU32};
use fugit_timer::Timer as TimerTrait;
use num_rational::Rational32;
use num_traits::{Inv, ToPrimitive};
use replace_with::replace_with_and_return;
use stepper::step_mode::StepMode;
use stepper::traits::{SetDirection, SetSleepMode, SetStepMode, Step};
use stepper::Direction;

pub use super::state::FsmStatus;
use super::state::State;

const DELAY_THRESHOLD_FOR_MICROSTEP: MillisDurationU32 = MillisDurationU32::from_ticks(20);

/// The finite state machine used to drive a stepper motor to point at the target as it
/// moves over time.
pub struct StepperVelocityController<Driver, Timer, const TIMER_HZ: u32>
where
    Driver: SetDirection + SetSleepMode + SetSleepMode + SetStepMode + Step,
    Timer: TimerTrait<TIMER_HZ>,
{
    next_velocity: Option<f64>,
    next_delay: Option<TimerDurationU32<TIMER_HZ>>,
    next_direction: Option<Direction>,
    next_step_mode: Option<Driver::StepMode>,
    current_velocity: f64,
    current_direction: Direction,
    current_step: Rational32,
    current_step_mode: Driver::StepMode,
    target_step: Option<f64>,
    state: State<Driver, Timer, TIMER_HZ>,
}

impl<Driver, Timer, const TIMER_HZ: u32> StepperVelocityController<Driver, Timer, TIMER_HZ>
where
    Driver: SetDirection + SetSleepMode + SetStepMode + Step,
    Timer: TimerTrait<TIMER_HZ>,
{
    pub fn new(driver: Driver, timer: Timer) -> Self {
        Self {
            next_velocity: None,
            next_delay: None,
            next_direction: None,
            next_step_mode: None,
            current_velocity: 0.0,
            current_direction: Direction::Forward,
            current_step: Rational32::new_raw(1, Driver::StepMode::MAX_STEP_BASE as i32),
            current_step_mode: 1.try_into().expect("Unable to convert into StepMode"),
            target_step: None,
            state: State::Idle {
                driver: driver,
                timer: timer,
            },
        }
    }

    pub fn step(&self) -> Rational32 {
        self.current_step
    }

    pub fn velocity(&self) -> f64 {
        self.current_velocity
    }

    /// Sets a step that this controller will attempt to step as close as possible to, without
    /// changing the provided velocity.
    ///
    /// This allows controllers that provide velocity to this one to more easily detect their
    /// exit conditions.
    pub fn set_target_step(&mut self, value: f64) {
        self.target_step = Some(value);
    }

    pub fn move_once_with_velocity(&mut self, velocity: f64) {
        self.next_velocity = Some(velocity);

        // See whether the direction has changed for this velocity
        let dir = direction_for_velocity(velocity);
        if dir != self.current_direction {
            self.next_direction = Some(dir);
        }

        // Determine the full-step delay for this velocity
        let mut delay = TimerDurationU32::<TIMER_HZ>::from_ticks(velocity.inv().abs() as u32);

        // See whether we can microstep to smooth things out
        let step_mode = self.find_microstep(delay, dir);
        if step_mode != self.current_step_mode {
            self.next_step_mode = Some(step_mode);
            let base = step_mode.into() as u32;
            delay /= base;
        }

        self.next_delay = Some(delay);
    }

    pub fn update(&mut self) -> Result<FsmStatus> {
        // Otherwise the closure will borrow all of `self`.
        let next_velocity = &mut self.next_velocity;
        let next_delay = &mut self.next_delay;
        let next_direction = &mut self.next_direction;
        let next_step_mode = &mut self.next_step_mode;
        let current_velocity = &mut self.current_velocity;
        let current_direction = &mut self.current_direction;
        let current_step = &mut self.current_step;
        let current_step_mode = &mut self.current_step_mode;

        replace_with_and_return(
            &mut self.state,
            || State::Invalid,
            |state| {
                super::state::update(
                    state,
                    next_velocity,
                    next_delay,
                    next_direction,
                    next_step_mode,
                    current_velocity,
                    current_direction,
                    current_step,
                    current_step_mode,
                )
            },
        )
    }

    fn find_microstep(
        &self,
        delay: TimerDurationU32<TIMER_HZ>,
        direction: Direction,
    ) -> Driver::StepMode {
        let dir_sign = direction as isize;

        // (self.current_step.to_f64().unwrap() - self.target_step.unwrap()).abs();

        Driver::StepMode::iter()
            .find(|step| {
                let step_denom: u16 = (*step).into();
                let step_delta = Rational32::new(dir_sign as i32, step_denom as i32);
                let next_step = self.current_step + step_delta;

                if let Some(target) = self.target_step {
                    step_range_contains(&self.current_step, &next_step, target);
                }

                let delay_with_microstep = delay / (step_denom as u32);

                // If we're below the threshold while at this step, then we've found what
                // we're looking for
                delay_with_microstep < DELAY_THRESHOLD_FOR_MICROSTEP
            })
            .unwrap_or(Driver::StepMode::MAX_STEP_BASE.try_into().unwrap())
    }
}

fn direction_for_velocity(velocity: f64) -> Direction {
    if velocity.is_sign_negative() {
        Direction::Backward
    } else {
        Direction::Forward
    }
}

fn step_range_contains(current: &Rational32, next: &Rational32, val: f64) -> bool {
    let (lower, upper) = if current < next {
        (current.to_f64().unwrap(), next.to_f64().unwrap())
    } else {
        (next.to_f64().unwrap(), current.to_f64().unwrap())
    };
    (lower..upper).contains(&val)
}

#[test]
fn foo() {
    let rng = 1.0..0.0;

    eprintln!("{:?}", rng);
    eprintln!("{:?}", rng.start);
    eprintln!("{:?}", rng.end);
}
