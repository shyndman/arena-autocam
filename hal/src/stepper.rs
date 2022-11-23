use num_traits::cast::ToPrimitive;

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
