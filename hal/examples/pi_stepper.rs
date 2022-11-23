use aa_hal::{
    clock::timer::Timer,
    prelude::*,
    stepper::FloatDelayToTicks,
    thread::{set_thread_as_realtime, sleep_nanos},
};
use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use stepper::{drivers::a4988::A4988, ramp_maker, Direction, Stepper};

const RATE_1MHZ: u32 = 1_000_000;

fn main() -> Result<()> {
    set_thread_as_realtime();

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .init();

    let gpio = Gpio::new()?;
    let step_pin = gpio.get(16)?.into_output();
    let direction_pin = gpio.get(21)?.into_output();
    let mut timer = Timer::<RATE_1MHZ>::new_non_blocking();

    let stepper = Stepper::from_driver(A4988::new())
        .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_control(step_pin);

    let profile = ramp_maker::Flat::<f32>::new();
    let mut motion_ctrl =
        stepper.enable_motion_control((timer.clone(), profile, FloatDelayToTicks));
    for degrees_per_second in (1..=20).map(|i| i as f32 * 5.0) {
        let degrees_per_tick = degrees_per_second / RATE_1MHZ as f32;
        info!(
            "ROTATE AT {}°/s ({}°/tick)",
            degrees_per_second, degrees_per_tick
        );
        let steps_per_tick = degrees_per_tick / 2.0;

        motion_ctrl
            .set_direction(Direction::Forward, &mut timer)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
        motion_ctrl
            .move_to_position(steps_per_tick, 30)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;

        info!("RESET");
        sleep_nanos(100 * 1_000_000); // Sleep 100ms

        // Reset to original position
        motion_ctrl
            .set_direction(Direction::Backward, &mut timer)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
        motion_ctrl
            .move_to_position(steps_per_tick, 0)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
    }

    Ok(())
}
