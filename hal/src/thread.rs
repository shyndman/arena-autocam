use std::time::Duration;

use libc::{sched_param, PR_SET_TIMERSLACK, SCHED_RR};

use crate::clock::get_time_ns;

const SLEEP_THRESHOLD: u64 = 250_000;
const BUSYWAIT_MAX: u64 = 200_000;
const BUSYWAIT_REMAINDER: i64 = 100;

/// Sleeps for `duration` as accurately as possible.
///
/// For best results, ensure the calling thread is using realtime priority, via
/// [`set_thread_as_realtime`].
#[inline(always)]
pub fn sleep(duration_ns: u64) -> i64 {
    let start_ns = get_time_ns();

    // Sleep if we have enough time remaining, while reserving some time
    // for busy waiting to compensate for sleep taking longer than needed.
    if duration_ns >= SLEEP_THRESHOLD {
        std::thread::sleep(Duration::from_nanos(duration_ns - BUSYWAIT_MAX));
    }

    // Busy-wait for the remaining active time, minus BUSYWAIT_REMAINDER
    // to account for get_time_ns() overhead
    loop {
        let elapsed_ns = get_time_ns() - start_ns;
        let remaining_ns: i64 = if elapsed_ns > duration_ns {
            (elapsed_ns - duration_ns) as i64 * -1
        } else {
            (duration_ns - elapsed_ns) as i64
        };
        if remaining_ns <= BUSYWAIT_REMAINDER {
            break remaining_ns * -1;
        }
    }
}

/// Sets the current thread as realitime priority
pub fn set_thread_as_realtime() {
    // Sets the current thread as the maximum priority of SCHED_RR.
    unsafe {
        let params = sched_param {
            sched_priority: libc::sched_get_priority_max(SCHED_RR),
        };
        libc::sched_setscheduler(0, SCHED_RR, &params);
    }

    // Set timer slack to 1 ns (default = 50 µs). This is only relevant if we're unable
    // to set a real-time scheduling policy.
    //
    // More information on timer slack: https://lwn.net/Articles/369549/
    unsafe {
        libc::prctl(PR_SET_TIMERSLACK, 1);
    }
}
