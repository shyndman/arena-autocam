pub struct FakeOutputPin(&'static str);

impl embedded_hal::digital::ErrorType for FakeOutputPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for FakeOutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        eprintln!("{}: 0", self.0);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        eprintln!("{}: 1", self.0);
        Ok(())
    }
}
