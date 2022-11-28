use aa_foundation::prelude::*;

#[derive(Default)]
pub struct FakeOutputPin {
    pub pin: u8,
    pub name: Option<String>,
}

impl embedded_hal::digital::ErrorType for FakeOutputPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for FakeOutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        trace!(target: "aa::gpio", "{}: 0", self.name.as_ref().unwrap_or(&self.pin.to_string()));
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        trace!(target: "aa::gpio", "{}: 1", self.name.as_ref().unwrap_or(&self.pin.to_string()));
        Ok(())
    }
}
