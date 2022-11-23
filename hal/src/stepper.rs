use anyhow::Result;
use embedded_hal::digital::OutputPin;
use num_traits::cast::ToPrimitive;

pub struct StepperPins<Pin>
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

#[cfg(target_arch = "aarch64")]
pub fn get_stepper_pins() -> Result<StepperPins<rppal::gpio::OutputPin>> {
    get_rpi_stepper_pins()
}

#[cfg(target_arch = "x86_64")]
pub fn get_stepper_pins() -> Result<StepperPins<crate::gpio::FakeOutputPin>> {
    get_fake_stepper_pins()
}

pub fn get_rpi_stepper_pins() -> Result<StepperPins<rppal::gpio::OutputPin>> {
    use rppal::gpio::Gpio;

    let gpio = Gpio::new()?;
    Ok(StepperPins {
        ms1_pin: gpio.get(26)?.into_output(),
        ms2_pin: gpio.get(19)?.into_output(),
        ms3_pin: gpio.get(13)?.into_output(),
        reset_pin: gpio.get(16)?.into_output(),
        sleep_pin: gpio.get(6)?.into_output(),
        step_pin: gpio.get(20)?.into_output(),
        direction_pin: gpio.get(21)?.into_output(),
    })
}

pub fn get_fake_stepper_pins() -> Result<StepperPins<crate::gpio::FakeOutputPin>> {
    use crate::gpio::FakeOutputPin;

    Ok(StepperPins {
        ms1_pin: FakeOutputPin(26),
        ms2_pin: FakeOutputPin(19),
        ms3_pin: FakeOutputPin(13),
        reset_pin: FakeOutputPin(16),
        sleep_pin: FakeOutputPin(6),
        step_pin: FakeOutputPin(20),
        direction_pin: FakeOutputPin(21),
    })
}

pub type DelayNum = fixed::FixedI64<typenum::U32>;

pub struct DelayToTicks;
impl<DelayUnit, const TIMER_HZ: u32>
    stepper::motion_control::DelayToTicks<DelayUnit, TIMER_HZ> for DelayToTicks
where
    DelayUnit: ToPrimitive,
{
    type Error = core::convert::Infallible;
    fn delay_to_ticks(
        &self,
        delay: DelayUnit,
    ) -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error> {
        let ticks = fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(
            delay.to_u32().expect("the delay to convert"),
        );
        Ok(ticks)
    }
}

pub struct FloatDelayToTicks;
impl<const TIMER_HZ: u32> stepper::motion_control::DelayToTicks<f32, TIMER_HZ>
    for FloatDelayToTicks
{
    type Error = core::convert::Infallible;
    fn delay_to_ticks(
        &self,
        delay: f32,
    ) -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error> {
        let ticks = fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(
            delay.to_u32().expect("the delay to convert"),
        );
        Ok(ticks)
    }
}
