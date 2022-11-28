//! The finite state machine responsible for driving a single stepper motor's target tracking
//! behaviour.
mod controller;
mod state;

pub use controller::StepperVelocityController;
pub use state::FsmStatus;
