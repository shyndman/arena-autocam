mod state;
use anyhow::{anyhow, Result};
use fugit_timer::Timer as TimerTrait;
use replace_with::replace_with_and_return;
use stepper::{
    motion_control::DelayToTicks,
    traits::{SetDirection, Step},
    Direction,
};

use self::state::State;

pub struct MovingTargetFollowerMotionControl<
    Driver,
    Timer,
    Convert,
    Delay,
    const TIMER_HZ: u32,
> {
    pub(self) state: State<Driver, Timer, Delay, TIMER_HZ>,
    pub(self) current_step: i32,
    pub(self) current_target_step: i32,
    pub(self) current_direction: Direction,
    pub(self) convert: Convert,
}

impl<Driver, Timer, Convert, Delay, const TIMER_HZ: u32>
    MovingTargetFollowerMotionControl<Driver, Timer, Convert, Delay, TIMER_HZ>
where
    Driver: SetDirection + Step,
    Timer: TimerTrait<TIMER_HZ>,
    Convert: DelayToTicks<Delay, TIMER_HZ>,
{
    pub fn new(driver: Driver, timer: Timer, convert: Convert) -> Self {
        Self {
            state: State::Idle { driver, timer },
            current_step: 0,
            current_target_step: 0,
            // Doesn't matter what we initialize it with. We're only using it
            // during an ongoing movement, and it will have been overridden at
            // that point.
            current_direction: Direction::Forward,
            convert,
        }
    }

    /// Access the current step
    pub fn current_step(&self) -> i32 {
        self.current_step
    }

    /// Access the current step
    pub fn current_target_step(&self) -> i32 {
        self.current_target_step
    }

    /// Access the current direction
    pub fn current_direction(&self) -> Direction {
        self.current_direction
    }

    fn update(&mut self) -> Result<bool> {
        // Otherwise the closure will borrow all of `self`.
        let current_step = &mut self.current_step;
        let current_target_step = &mut self.current_target_step;
        let current_direction = &mut self.current_direction;
        let convert = &self.convert;

        replace_with_and_return(
            &mut self.state,
            || State::Invalid,
            |state| {
                state::update(
                    state,
                    current_step,
                    current_target_step,
                    current_direction,
                    convert,
                )
            },
        )
        .map_err(|_| anyhow!(""))
    }
}
