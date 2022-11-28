use aa_foundation::thread::set_thread_timerslack;
use aa_sys::{pantilt::hal::PanStepperPinMapping, timer::make_software_timer};
#[allow(unused)]
use anyhow::{anyhow, Result};
use stepper::{drivers::a4988::A4988, Stepper};

fn main() -> Result<()> {
    aa_foundation::tracing::setup_dev_tracing_subscriber();
    set_thread_timerslack(1);

    let PanStepperPinMapping { sleep_pin, .. } =
        aa_sys::pantilt::hal::get_rpi_stepper_pins()?;

    let mut timer = make_software_timer();
    let mut stepper = Stepper::from_driver(A4988::new()).enable_sleep_mode_control(sleep_pin);

    stepper.sleep(&mut timer).wait().unwrap();

    Ok(())
}
