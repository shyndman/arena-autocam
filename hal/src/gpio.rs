use aa_foundation::prelude::*;

pub struct FakeOutputPin(pub u8);

impl embedded_hal::digital::ErrorType for FakeOutputPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for FakeOutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        trace!("{}: 0", self.0);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        trace!("{}: 1", self.0);
        Ok(())
    }
}
