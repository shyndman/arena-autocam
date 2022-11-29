mod timer;

use aa_foundation::thread::get_thread_timerslack;
use aa_foundation::trace_category;
use timer::Timer;

trace_category!("timer");

pub const RATE_1MHZ: u32 = 1_000_000;

pub fn make_software_timer() -> Timer<RATE_1MHZ> {
    assert!(
        get_thread_timerslack() == 1,
        "Thread not configured for the software timer. Call set_thread_timerslack(1)"
    );

    Timer::new_non_blocking()
}
