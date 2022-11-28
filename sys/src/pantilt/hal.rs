use anyhow::Result;
use embedded_hal::digital::OutputPin;
use stepper::{drivers::a4988::A4988, traits::*};

pub fn create_pan_stepper() -> Result<A4988<(), Pin, Pin, Pin, Pin, Pin, Pin, Pin>> {
    Ok(create_pan_driver(get_stepper_pins()?))
}

pub fn get_stepper_pins() -> Result<PanStepperPinMapping<Pin>> {
    #[cfg(target_arch = "aarch64")]
    {
        get_rpi_stepper_pins()
    }
    #[cfg(target_arch = "x86_64")]
    {
        get_fake_stepper_pins()
    }
}

fn create_pan_driver<Pin>(
    pins: PanStepperPinMapping<Pin>,
) -> A4988<(), Pin, Pin, Pin, Pin, Pin, Pin, Pin>
where
    Pin: OutputPin + Sized,
{
    let PanStepperPinMapping {
        ms1_pin,
        ms2_pin,
        ms3_pin,
        reset_pin,
        sleep_pin,
        step_pin,
        direction_pin,
    } = pins;

    A4988::new()
        .enable_step_control(step_pin)
        .enable_direction_control(direction_pin)
        .enable_step_mode_control((reset_pin, ms1_pin, ms2_pin, ms3_pin))
        .enable_sleep_mode_control(sleep_pin)
}

#[cfg(target_arch = "aarch64")]
type Pin = crate::gpio::FakeOutputPin;

#[cfg(target_arch = "x86_64")]
type Pin = crate::gpio::fake::FakeOutputPin;

pub struct PanStepperPinMapping<Pin>
where
    Pin: OutputPin + Sized,
{
    pub step_pin: Pin,
    pub direction_pin: Pin,
    pub sleep_pin: Pin,
    pub reset_pin: Pin,
    pub ms1_pin: Pin,
    pub ms2_pin: Pin,
    pub ms3_pin: Pin,
}

impl<Pin> PanStepperPinMapping<Pin>
where
    Pin: OutputPin + Sized,
{
    fn new<F>(build_pin: F) -> Self
    where
        F: Fn(u8, &'static str) -> Pin,
    {
        PanStepperPinMapping {
            ms1_pin: build_pin(26, "ms1"),
            ms2_pin: build_pin(19, "ms2"),
            ms3_pin: build_pin(13, "ms3"),
            reset_pin: build_pin(16, "reset"),
            sleep_pin: build_pin(6, "sleep"),
            step_pin: build_pin(20, "step"),
            direction_pin: build_pin(21, "direction"),
        }
    }
}

#[allow(unused)]
pub fn get_rpi_stepper_pins() -> Result<PanStepperPinMapping<rppal::gpio::OutputPin>> {
    use rppal::gpio::Gpio;

    let gpio = Gpio::new()?;
    Ok(PanStepperPinMapping::new(move |pin, name| {
        gpio.get(pin)
            .expect("Could not get pin from GPIO")
            .into_output()
    }))
}

#[allow(unused)]

pub fn get_fake_stepper_pins(
) -> Result<PanStepperPinMapping<crate::gpio::fake::FakeOutputPin>> {
    use crate::gpio::fake::FakeOutputPin;

    Ok(PanStepperPinMapping::new(|pin, name| FakeOutputPin {
        pin: 26,
        name: Some(name.to_string()),
    }))
}
