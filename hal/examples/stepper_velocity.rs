use aa_hal::{
    clock::timer::Timer,
    prelude::*,
    stepper::FloatDelayToTicks,
    thread::{set_thread_as_realtime, sleep_nanos},
};
use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use stepper::{drivers::a4988::A4988, ramp_maker, step_mode::StepMode16, Direction, Stepper};

const RATE_1MHZ: u32 = 1_000_000;

fn main() -> Result<()> {
    set_thread_as_realtime();

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .init();

    let gpio = Gpio::new()?;
    let step_pin = gpio.get(20)?.into_output();
    let direction_pin = gpio.get(21)?.into_output();

    let reset_pin = gpio.get(16)?.into_output();
    let ms1_pin = gpio.get(26)?.into_output();
    let ms2_pin = gpio.get(19)?.into_output();
    let ms3_pin = gpio.get(13)?.into_output();

    let mut timer = Timer::<RATE_1MHZ>::new_blocking();
    let mut stepper = Stepper::from_driver(A4988::new())
        .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_mode_control(
            (reset_pin, ms1_pin, ms2_pin, ms3_pin),
            StepMode16::Full,
            &mut timer,
        )
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_control(step_pin)
        .enable_motion_control((
            timer.clone(),
            ramp_maker::Flat::<f32>::new(),
            FloatDelayToTicks,
        ));

    for degs_per_second in (10..=30).map(|i| i as f32 * 4.0) {
        let degs_per_tick = degs_per_second / RATE_1MHZ as f32;
        info!("ROTATE AT {}°/s ({}°/tick)", degs_per_second, degs_per_tick);
        let steps_per_tick = degs_per_tick / 2.0;

        stepper
            .set_direction(Direction::Forward, &mut timer)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
        stepper
            .move_to_position(steps_per_tick, 30)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;

        info!("RESET");
        sleep_nanos(100 * 1_000_000); // Sleep 100ms

        // Reset to original position
        stepper
            .set_direction(Direction::Backward, &mut timer)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
        sleep_nanos(2 * 1_000); // Sleep 2µs
        stepper
            .move_to_position(steps_per_tick, 0)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
    }

    Ok(())
}
