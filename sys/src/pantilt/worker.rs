use std::thread::JoinHandle;

use aa_foundation::spring::{
    update_spring_system, SpringConfig, SpringSystemState, SpringsUpdateResult,
};
use aa_foundation::thread::set_thread_timerslack;
use anyhow::Result;
use crossbeam::channel::Sender;

use super::hal::create_pan_stepper;
use super::tracing::*;
use super::PanTiltCommand;
use crate::stepper::velocity::{FsmStatus, StepperVelocityController};
use crate::timer::make_software_timer;

pub fn start_worker_thread() -> Result<(JoinHandle<()>, Sender<PanTiltCommand>)> {
    let (send_channel, receive_channel) = crossbeam::channel::unbounded();
    let join_handle =
        std::thread::Builder::new()
            .name("pantilt".into())
            .spawn(move || {
                thread_main(receive_channel)
                    .expect("The pantilt control thread encountered an error");
            })?;
    Ok((join_handle, send_channel))
}

fn minimize_timerslack() {
    set_thread_timerslack(1);
}

fn thread_main(cmd_channel: crossbeam::channel::Receiver<PanTiltCommand>) -> Result<()> {
    info!("starting pantilt worker thread");
    minimize_timerslack();

    let timer = make_software_timer();
    let mut spring_state = SpringSystemState::from_time_provider(&timer);
    spring_state.spring_config = SpringConfig {
        clamp: true,
        friction: 40.0,
        max_velocity: Some(0.001),
        ..SpringConfig::stiff()
    };
    let mut velocity_ctrl = StepperVelocityController::new(create_pan_stepper()?, timer);

    loop {
        // First let's check whether we've received any commands since the previous iteration
        // of the loop
        if let Ok(cmd) = cmd_channel.try_recv() {
            match cmd {
                PanTiltCommand::UpdateTarget { target_value } => {
                    debug!(target_value, "received target from channel");
                    spring_state.update_target_value(target_value);
                    velocity_ctrl.set_target_step(target_value);
                }
            }
        }

        // Attempt to update controller state machine. If the state machine did not complete
        // its update, or an error occurred, we keep looping.
        if !match velocity_ctrl.update() {
            Ok(FsmStatus::Ready) => {
                let step = velocity_ctrl.step();
                let step_float = *step.numer() as f64 / *step.denom() as f64;
                let velocity = velocity_ctrl.velocity();
                spring_state.apply_state_updates(step_float, velocity);

                debug!(
                    value = step_float,
                    velocity,
                    distance = spring_state.distance_to_target(),
                    from = spring_state.from_value,
                    target = spring_state.target_value,
                    "step complete. values applied to hardware."
                );

                true
            }
            Ok(_) => false,
            Err(err) => {
                error!("{:?}", err);
                false
            }
        } {
            continue;
        }

        match update_spring_system(&spring_state) {
            SpringsUpdateResult::VelocityChanged { new_velocity } => {
                trace!(new_velocity, "spring velocity change");
                velocity_ctrl.move_once_with_velocity(new_velocity);
            }
            SpringsUpdateResult::Finished { position } => {
                trace!(position, "spring system is complete for now");
                panic!();
                // velocity_ctrl.move_to_rest(position);
            }
        }
    }
}
