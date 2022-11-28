use fugit::Duration;

pub trait DurationExtTrait {
    fn increment(&self) -> Self;
    fn decrement(&self) -> Self;
}

impl<const DENOM: u32> DurationExtTrait for Duration<u32, 1, DENOM> {
    fn increment(&self) -> Self {
        Self::from_ticks(self.ticks() + 1)
    }
    fn decrement(&self) -> Self {
        Self::from_ticks(self.ticks() - 1)
    }
}

impl<const DENOM: u32> DurationExtTrait for Duration<u64, 1, DENOM> {
    fn increment(&self) -> Self {
        Self::from_ticks(self.ticks() + 1)
    }
    fn decrement(&self) -> Self {
        Self::from_ticks(self.ticks() - 1)
    }
}
