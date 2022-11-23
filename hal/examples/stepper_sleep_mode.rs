// #[allow(unused)]
// use aa_foundation::prelude::*;
// use aa_hal::{
//     clock::timer::Timer,
//     stepper::{get_stepper_pins, FloatDelayToTicks, StepperPins, get_rpi_stepper_pins},
//     thread::set_thread_as_realtime,
// };
// use anyhow::{anyhow, Result};
// use embedded_hal::digital::OutputPin;
// use stepper::{drivers::a4988::A4988, ramp_maker, step_mode::StepMode16, Direction, Stepper};

// const RATE_1MHZ: u32 = 1_000_000;

// fn main() -> Result<()> {
//     aa_foundation::tracing::setup_dev_tracing_subscriber();
//     set_thread_as_realtime();

//     let StepperPins {
//         ms1_pin,
//         ms2_pin,
//         ms3_pin,
//         reset_pin,
//         sleep_pin,
//         step_pin,
//         direction_pin,
//     } = get_rpi_stepper_pins()?;

//     let mut timer = Timer::<RATE_1MHZ>::new_non_blocking();
//     let mut stepper = Stepper::from_driver(A4988::new())
//     .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
//     .map_err(|e| anyhow!("{:?}", e))?
//     .enable_sleep_mode_control(sleep_pin);
//     // .enable_step_mode_control(
//     //     (reset_pin, ms1_pin, ms2_pin, ms3_pin),
//     //     StepMode16::Full,
//     //     &mut timer,
//     // )
//     // .map_err(|e| anyhow!("{:?}", e))?
//     // .enable_step_control(step_pin)
//     // .enable_motion_control((
//     //     timer.clone(),
//     //     ramp_maker::Flat::<f32>::new(),
//     //     FloatDelayToTicks,
//     // ));

//     Ok(())
// }

fn main() {}
