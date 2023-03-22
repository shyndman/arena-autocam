//! Responsible for setting up the systems necessary for communication with hardware

use aa_foundation::trace_category;
use aa_sys::pantilt::PanTiltSystem;
use anyhow::Result;

use self::tracing::*;

trace_category!("app::system");

pub fn init_hardware_systems() -> Result<HardwareSystems> {
    info!("Initializing hardware systems");

    let pantilt = PanTiltSystem::init_system()?;

    Ok(HardwareSystems { pantilt })
}

pub struct HardwareSystems {
    pub pantilt: PanTiltSystem,
}
