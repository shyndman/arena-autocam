use num_traits::cast::ToPrimitive;

pub type DelayNum = fixed::FixedI64<typenum::U32>;

pub struct DelayToTicks;
impl<const TIMER_HZ: u32> stepper::motion_control::DelayToTicks<DelayNum, TIMER_HZ>
    for DelayToTicks
{
    type Error = core::convert::Infallible;
    fn delay_to_ticks(
        &self,
        delay: DelayNum,
    ) -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error> {
        let ticks = fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(
            DelayNum::to_u32(&delay).expect("the delay to convert"),
        );
        Ok(ticks)
    }
}
