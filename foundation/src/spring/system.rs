use approx::relative_eq;

use super::tracing::*;
use super::SpringConfig;

pub trait SpringSystemRateProvider<const RATE: u32> {}

/// Indicates how a spring system has changed as a result of running [`update_system`]
pub enum SpringsUpdateResult<const RATE: u32> {
    /// Indicates the system is still moving, and provides the new velocity
    VelocityChanged { new_velocity: f64 },
    /// Indicates the simulation has reached a rest state
    Finished { position: f64 },
}

/// A snapshot of a spring system's state.
///
/// You can imagine the system as a spring stretched between two points. One end is fixed and
/// attached to a point at [`to_value`], while the other end is movable and has been stretched
/// to [`from_value`].
///
/// The state's [`current_velocity`] and [`current_value`] fields describe the movable end's
/// motion at a point in time.
#[derive(Default, Clone)]
pub struct SpringSystemState<const RATE: u32> {
    /// The starting value
    pub from_value: f64,
    /// The destination value
    pub target_value: f64,

    /// The current value of the simulation
    pub value: f64,
    /// The current velocity of the value
    pub velocity: f64,
    /// Details about the spring (its tension, friction, etc)
    pub spring_config: SpringConfig,

    /// The number of ticks that have elapsed in the simulation
    is_first_tick: bool,
}

impl<const RATE: u32> SpringSystemState<RATE> {
    pub fn from_time_provider<T: SpringSystemRateProvider<RATE>>(_: &T) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn apply_state_updates(&mut self, value: f64, velocity: f64) {
        self.value = value;
        self.velocity = velocity;
        self.is_first_tick = false;
    }

    pub fn update_target_value(&mut self, value: f64) {
        self.target_value = value;
    }

    pub fn is_first_tick(&self) -> bool {
        self.is_first_tick
    }

    pub fn velocity(&self) -> f64 {
        if self.is_first_tick() {
            self.spring_config.initial_velocity()
        } else {
            self.velocity
        }
    }

    pub fn distance_to_target(&self) -> f64 {
        (self.value - self.target_value).abs()
    }
}

const SIMULATION_STEPS_PER_UPDATE: u32 = 20;

/// Updates the provided spring system state by reference.
pub fn update_spring_system<const RATE: u32>(
    state: &SpringSystemState<RATE>,
) -> SpringsUpdateResult<RATE> {
    trace!("updating spring system");

    let mut value = state.value;
    let mut velocity = state.velocity();

    let config = &state.spring_config;
    let max_velocity = config.max_velocity();
    let RateScaledSystemConfig {
        precision,
        rest_velocity,
        tension,
        friction,
    } = get_rate_scaled_config_values(state);

    trace!(
        %tension,
        %friction,
        %precision,
        %rest_velocity,
        "scaled config for RATE={}",
        RATE
    );

    for _ in 0..SIMULATION_STEPS_PER_UPDATE {
        let distance_from_target = value - state.target_value;

        // Determine whether we have stopped moving, or are moving so little as to
        // be considered stopped
        let has_stopped = velocity.abs() <= rest_velocity;
        let at_target = distance_from_target.abs() <= precision;
        if has_stopped && at_target {
            return SpringsUpdateResult::Finished {
                position: state.target_value,
            };
        }

        let spring_force = -tension * distance_from_target;
        let damping_force = -friction * velocity;
        let acceleration = (spring_force + damping_force) / config.mass;

        velocity = (velocity + acceleration).clamp(-max_velocity, max_velocity);
        value += velocity;
    }

    SpringsUpdateResult::VelocityChanged {
        new_velocity: velocity,
    }
}

fn get_rate_scaled_config_values<const RATE: u32>(
    state: &SpringSystemState<RATE>,
) -> RateScaledSystemConfig<RATE> {
    let precision = if let Some(precision_override) = state.spring_config.precision {
        precision_override
    } else if relative_eq!(state.from_value, state.target_value) {
        COINCIDENT_FROM_TO_PRECISION
    } else {
        (state.target_value - state.from_value).abs().min(1.0)
    };
    let rest_velocity = precision / 10.0;

    RateScaledSystemConfig::<RATE>::from_unscaled(
        precision,
        rest_velocity,
        state.spring_config.tension,
        state.spring_config.friction,
    )
}

/// Scales the impact of spring tension on the system.
///
/// This value should be further reduced by the system's update rate.
const TENSION_SCALE_BASE: f64 = 1E-3;
/// Scales the impact of friction on the system.
///
/// This value should be further reduced by the system's update rate.
const FRICTION_SCALE_BASE: f64 = 1.0;
/// The precision value used when its from and to values are the same.
///
/// This value should be scaled by the system's update rate.
const COINCIDENT_FROM_TO_PRECISION: f64 = 5.0;

/// A set of system configuration values that have been scaled by the system's update rate
/// (Hz).
struct RateScaledSystemConfig<const RATE: u32> {
    precision: f64,
    rest_velocity: f64,
    tension: f64,
    friction: f64,
}

impl<const RATE: u32> RateScaledSystemConfig<RATE> {
    fn from_unscaled(
        precision: f64,
        rest_velocity: f64,
        tension: f64,
        friction: f64,
    ) -> Self {
        Self {
            precision: precision / RATE as f64,
            rest_velocity: rest_velocity / RATE as f64,
            tension: tension * TENSION_SCALE_BASE / RATE as f64,
            friction: friction * FRICTION_SCALE_BASE / RATE as f64,
        }
    }
}

#[cfg(test)]
mod test {
    use super::update_spring_system;
    use crate::spring::SpringConfig;
    use crate::tracing::setup_dev_tracing_subscriber;

    #[test]
    fn test_system() {
        use super::super::tracing::*;
        info!("hi hi hi");

        setup_dev_tracing_subscriber();
        let state = super::SpringSystemState::<1_000_000> {
            from_value: 0.0,
            target_value: 4.0,
            value: 0.0,
            velocity: 0.0,
            spring_config: SpringConfig::default(),
            is_first_tick: true,
        };
        update_spring_system(&state);
    }
}
