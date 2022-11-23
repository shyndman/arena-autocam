use aa_hal::{
    clock::timer::non_blocking::Timer,
    stepper::{DelayNum, DelayToTicks},
    thread::set_thread_as_realtime,
};
use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use stepper::{drivers::a4988::A4988, ramp_maker, Direction, MoveToFuture, Stepper};

const RATE_1MHZ: u32 = 1_000_000;

fn main() -> Result<()> {
    set_thread_as_realtime();

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    let gpio = Gpio::new()?;
    let step_pin = gpio.get(16)?.into_output();
    let direction_pin = gpio.get(21)?.into_output();
    let mut timer = Timer::<RATE_1MHZ>::new();

    let stepper = Stepper::from_driver(A4988::new())
        .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_control(step_pin);


    let max_steps_per_second = 10;
    let target_accel = DelayNum::from(max_steps_per_second) / (1_000_000 * 6);
    let max_speed = DelayNum::from(max_steps_per_second) / 1_000_000;
    let profile = ramp_maker::Trapezoidal::new(target_accel);
    let mut stepper_motion = stepper.enable_motion_control((timer, profile, DelayToTicks));

    // This is responsible for moving the animation forward. If we want to stop the
    // animation, we can release and it's as simple as that.
    let mut move_to_future: MoveToFuture<_> =
        stepper_motion.move_to_position(max_speed, 200);
    move_to_future.wait().map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}
