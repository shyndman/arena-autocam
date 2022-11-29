//! This crate handles the setup and control of the project's panning and tilting motors.

pub mod hal;
mod worker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

#[allow(unused)]
use aa_foundation::prelude::*;
use anyhow::{ensure, Result};
use crossbeam::channel::Sender;

/// Used to instruct the pantilt system where it should be pointing
pub struct PanTiltController {
    join_handle: Option<JoinHandle<()>>,
    send_channel: Sender<PanTiltCommand>,
}

impl PanTiltController {
    /// Note, this method can only be called once.
    pub fn init_system_controller() -> Result<Self> {
        static CREATED: AtomicBool = AtomicBool::new(false);
        ensure!(
            !CREATED.fetch_or(true, Ordering::Relaxed),
            "Only one PantiltController can be created per application"
        );

        let (join_handle, send_channel) = worker::start_worker_thread()?;
        Ok(Self {
            join_handle: Some(join_handle),
            send_channel,
        })
    }

    pub fn update_target(&self, target_value: f64) -> Result<()> {
        self.send_channel
            .send(PanTiltCommand::UpdateTarget { target_value })?;
        Ok(())
    }

    pub fn join(mut self) -> Result<()> {
        if let Some(handle) = self.join_handle.take() {
            handle.join().unwrap();
        }
        Ok(())
    }
}

/// The commands sent to the
pub enum PanTiltCommand {
    UpdateTarget { target_value: f64 },
}

#[allow(unused)]
pub mod trace {
    use aa_foundation::trace_macros_for_target;
    trace_macros_for_target!("pantilt");
    pub(crate) use {debug, error, info, trace, warning};
}
