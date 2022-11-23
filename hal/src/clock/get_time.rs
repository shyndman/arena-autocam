use libc::{timespec, CLOCK_MONOTONIC};

const NANOS_PER_SEC: u64 = 1_000_000_000;

#[inline(always)]
pub fn get_time_ns() -> u64 {
    let mut ts = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        libc::clock_gettime(CLOCK_MONOTONIC, &mut ts);
    }

    (ts.tv_sec as u64 * NANOS_PER_SEC) + ts.tv_nsec as u64
}
