use anyhow::Result;
use rust_pigpio as pigpio;

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    println!(
        "Initialized pigpio. Version: {}",
        pigpio::initialize().unwrap()
    );
    eprintln!("{}us", pigpio::delay(30));

    pigpio::terminate();

    Ok(())
}
