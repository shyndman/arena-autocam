use std::time::Duration;

use aa_hal::{
    clock::get_time_ns,
    thread::{set_thread_as_realtime, sleep_nanos},
};
use num_format::{CustomFormat, ToFormattedString};

const SLEEP_DURATIONS: [Duration; 21] = [
    Duration::from_nanos(300_000),
    Duration::from_nanos(250_000),
    Duration::from_nanos(200_000),
    Duration::from_nanos(150_000),
    Duration::from_nanos(100_000),
    Duration::from_nanos(50_000),
    Duration::from_nanos(25_000),
    Duration::from_nanos(10_000),
    Duration::from_nanos(5_000),
    Duration::from_nanos(2_000),
    Duration::from_nanos(1_000),
    Duration::from_nanos(500),
    Duration::from_nanos(400),
    Duration::from_nanos(300),
    Duration::from_nanos(200),
    Duration::from_nanos(100),
    Duration::from_nanos(50),
    Duration::from_nanos(40),
    Duration::from_nanos(30),
    Duration::from_nanos(20),
    Duration::from_nanos(10),
];

fn main() {
    eprintln!("PRIORITY: NORMAL");
    run_sleeps();
    measure_volatile_rw_duration(10);
    measure_volatile_rw_duration(100);
    measure_volatile_rw_duration(1000);

    eprintln!("PRIORITY: REALTIME");
    set_thread_as_realtime();
    run_sleeps();
    measure_volatile_rw_duration(10);
    measure_volatile_rw_duration(100);
    measure_volatile_rw_duration(1000);
}

fn run_sleeps() {
    let fmt = CustomFormat::builder()
        .separator("_")
        .plus_sign("+")
        .build()
        .unwrap();
    for d in SLEEP_DURATIONS {
        let s_ns = d.as_nanos() as u64;
        let start_ns = get_time_ns();
        let r_ns: i64 = sleep_nanos(s_ns);
        let d_ns = get_time_ns() - start_ns;
        eprintln!("sleep {}ns", s_ns.to_formatted_string(&fmt));
        eprintln!(
            "  dur {}ns \n  returned at {:+}ns\n  diff {:+}ns",
            d_ns.to_formatted_string(&fmt),
            r_ns.to_formatted_string(&fmt),
            (d_ns as i64 - s_ns as i64).to_formatted_string(&fmt)
        );
    }
}

fn measure_volatile_rw_duration(count: u32) {
    let start_ns = get_time_ns();
    let mut i = 0;
    unsafe {
        // Volatile writes are slow enough to count down the time
        // until the next tick
        for _ in 0..count {
            std::ptr::write_volatile(&mut i, std::ptr::read_volatile(&mut i) + 1);
        }
    }
    let elapsed_ns = get_time_ns() - start_ns;

    eprintln!(
        "volatile RW x{}: {}ns ({}ns / op)",
        count,
        elapsed_ns,
        elapsed_ns as f64 / count as f64
    );
}
