use aa_foundation::prelude::*;
use fugit::TimerDurationU32;

type Micros = TimerDurationU32<1_000_000>;
type Nanos = TimerDurationU32<1_000_000_000>;

fn main() {
    let us = Micros::from_ticks(5);
    let ns2 = Nanos::from_ticks(1500);

    let mut res = us - ns2.convert();
    if res == us {
        res = us.increment();
    }

    println!("{}", res);
}
