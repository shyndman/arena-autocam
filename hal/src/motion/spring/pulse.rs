#[allow(unused_imports)]
use aa_foundation::prelude::*;
use fugit::Duration;
use num_traits::Inv;

use super::SpringConfig;
pub type DelayNum = fixed::FixedI64<typenum::U32>;

type DurationU64<const TICKS_PER_SECOND: u32> = Duration<u64, 1, TICKS_PER_SECOND>;

pub struct Pulse<const TICKS_PER_SECOND: u32> {
    pub delay: DurationU64<TICKS_PER_SECOND>,
    pub current_value: f64,
    pub next_value: f64,
    pub velocity: f64,
    pub acceleration: f64,
}

/// Describes the current state of a spring
pub struct Spring<const TICKS_PER_SECOND: u32> {
    /// The spring's origin value
    from_value: f64,
    /// The spring's destination value
    to_value: f64,
    /// The previous delay value we returned to the caller
    current_delay: DurationU64<TICKS_PER_SECOND>,
    /// The value of the spring after the currently outstanding delay elapses
    next_value: f64,
    /// All velocities have the unit of steps / tick, where tick is
    max_velocity: f64,
    current_velocity: Option<f64>,
}

impl<const TICKS_PER_SECOND: u32> Default for Spring<TICKS_PER_SECOND> {
    fn default() -> Self {
        Self {
            next_value: 0.0,
            from_value: 0.0,
            to_value: 0.0,
            max_velocity: 0.0,
            current_velocity: None,
            current_delay: DurationU64::from_ticks(0),
        }
    }
}

const TENSION_SCALE: f64 = 1E-10;
const FRICTION_SCALE: f64 = 1E-3;
const DEFAULT_PRECISION: f64 = 5.0;

// Duration

impl<const TICKS_PER_SECOND: u32> Spring<TICKS_PER_SECOND> {
    pub fn new(from_value: f64, to_value: f64, max_velocity: f64) -> Self {
        Self {
            next_value: from_value,
            from_value: from_value,
            to_value: to_value,
            max_velocity: max_velocity,
            ..Default::default()
        }
    }

    pub fn update_target_by(&mut self, rel_to_value: f64) {
        self.from_value = self.next_value;
        self.to_value += rel_to_value;
        self.current_velocity = Some(0.0);
        self.current_delay = DurationU64::from_ticks(0);
    }

    pub fn next_pulse(
        &mut self,
        update: Option<(f64, DurationU64<TICKS_PER_SECOND>)>,
        config: &SpringConfig,
    ) -> Option<Pulse<TICKS_PER_SECOND>> {
        if let Some((next_value, current_delay)) = update {
            self.next_value = next_value;
            self.current_delay = current_delay;
        }

        let v0 = config.velocity.unwrap_or(0.0);
        let prev_velocity = self.current_velocity.unwrap_or(v0);
        let current_value = self.next_value;
        let mut velocity = prev_velocity;
        let mut acceleration = 0.0;

        // Determine whether we have stopped moving, or are moving so little as to
        // be considered stopped
        let precision = config.precision.unwrap_or_else(|| {
            if self.from_value == self.to_value {
                DEFAULT_PRECISION / TICKS_PER_SECOND as f64
            } else {
                ((self.to_value - self.from_value).abs() / TICKS_PER_SECOND as f64).min(1.0)
            }
        });
        let rest_velocity = precision / 10.0;
        trace!("precision={precision:.8}, rest_velocity={rest_velocity:.8}");

        for _i in 0..10 {
            let distance_from_value = (self.to_value - self.next_value).abs();
            let is_moving = velocity.abs() > rest_velocity;
            trace!("is_moving={is_moving}, distance_from_value={distance_from_value}");
            if !is_moving {
                let finished = precision >= distance_from_value;
                if finished {
                    trace!("spring finished diff={:.8}", distance_from_value);
                    return None;
                }
            }

            // Bouncing is opt-in (not to be confused with overshooting)
            let bounce_factor = config.bounce_factor();
            if bounce_factor != 0.0 {
                let is_growing = if self.from_value == self.to_value {
                    v0 > 0.0
                } else {
                    self.to_value > self.from_value
                };
                let is_bouncing = self.next_value == self.to_value ||
                    (self.next_value > self.to_value) == is_growing;

                // Invert the velocity with a magnitude, or clamp it.
                if is_bouncing {
                    velocity = -velocity * bounce_factor;
                    self.next_value = self.to_value;
                }
            }

            let spring_force =
                -config.tension * TENSION_SCALE * (self.next_value - self.to_value);
            let damping_force = -config.friction * FRICTION_SCALE * velocity;
            acceleration = (spring_force + damping_force) / config.mass;

            velocity = (velocity + acceleration).clamp(-self.max_velocity, self.max_velocity);
            self.next_value += velocity;
        }

        self.current_velocity = Some(velocity);
        self.next_value = current_value + velocity.signum() * 1.0;
        self.current_delay = DurationU64::from_ticks(velocity.inv().abs() as u64);

        Some(Pulse {
            delay: self.current_delay,
            current_value,
            next_value: self.next_value,
            velocity,
            acceleration: acceleration,
        })
    }
}
