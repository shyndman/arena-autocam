use aa_foundation::thread::set_thread_timerslack;
use aa_sys::pantilt::hal::PanStepperPinMapping;
use aa_sys::timer::make_software_timer;
#[allow(unused)]
use anyhow::{anyhow, Result};
use stepper::drivers::a4988::A4988;
use stepper::step_mode::StepMode16;
use stepper::Stepper;

fn main() -> Result<()> {
    aa_foundation::tracing::setup_dev_tracing_subscriber();
    set_thread_timerslack(1);

    let PanStepperPinMapping {
        step_pin,
        direction_pin,
        reset_pin,
        ms1_pin,
        ms2_pin,
        ms3_pin,
        ..
    } = aa_sys::pantilt::hal::get_rpi_stepper_pins()?;

    let mut timer = make_software_timer();
    let mut stepper = Stepper::from_driver(A4988::new())
    .enable_step_control(step_pin)
        .enable_direction_control(
            direction_pin,
            stepper::Direction::Forward,
            &mut timer.clone(),
        )
        .unwrap()
        .enable_step_mode_control(
            (reset_pin, ms1_pin, ms2_pin, ms3_pin),
            StepMode16::Full,
            &mut timer,
        ).unwrap();

    Ok(())
}
