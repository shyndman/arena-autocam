use std::error::Error;

use anyhow::Result;
use rppal::{gpio::Gpio, system::DeviceInfo};

fn main() -> Result<(), Box<dyn Error>> {
    let device_info = DeviceInfo::new()?;
    println!("Device information {:?}", device_info);
    for pin in (2..26).map(|i| {
        let pin = Gpio::new().unwrap();
        match pin.get(i) {
            Ok(pin) => Some(pin),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }) {
        if let Some(p) = pin {
            eprintln!(
                "{:>2}: {:>5} {} ({:?})",
                p.pin(),
                p.mode().to_string(),
                p.read(),
                p
            );
        }
    }
    Ok(())
}
