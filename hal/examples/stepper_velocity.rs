use aa_foundation::prelude::*;
use aa_hal::{
    clock::timer::Timer,
    stepper::{get_stepper_pins, FloatDelayToTicks, StepperPins},
    thread::set_thread_as_realtime,
};
use anyhow::{anyhow, Result};
use stepper::{drivers::a4988::A4988, ramp_maker, step_mode::StepMode16, Direction, Stepper};

const RATE_1MHZ: u32 = 1_000_000;

fn main() -> Result<()> {
    aa_foundation::tracing::setup_dev_tracing_subscriber();
    set_thread_as_realtime();

    let StepperPins {
        ms1_pin,
        ms2_pin,
        ms3_pin,
        reset_pin,
        sleep_pin: _,
        step_pin,
        direction_pin,
    } = get_stepper_pins()?;

    let mut timer = Timer::<RATE_1MHZ>::new_non_blocking();
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

    let reset_velocity = velocity_in_steps_per_tick(360.0);
    for degs_per_second in (10..=40).map(|i| i as f32 * 40.0) {
        let velocity = velocity_in_steps_per_tick(degs_per_second);
        info!(
            "ROTATE AT {:>3}°/s ({:.7} steps/tick)",
            degs_per_second, velocity
        );

        stepper
            .move_to_position(velocity, 200)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;

        // Reset to original position

        stepper
            .move_to_position(reset_velocity, 0)
            .wait()
            .map_err(|e| anyhow!("{:?}", e))?;
    }

    Ok(())
}

fn velocity_in_steps_per_tick(degrees_per_second: f32) -> f32 {
    let degrees_per_tick = degrees_per_second / RATE_1MHZ as f32;
    degrees_to_steps(degrees_per_tick)
}

const DEGREES_PER_STEP: f32 = 360.0 / 200.0;

fn degrees_to_steps(degrees: f32) -> f32 {
    degrees / DEGREES_PER_STEP
}
