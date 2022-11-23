pub struct SpringConfig {
    pub tension: f32,
    pub friction: f32,
    pub mass: f32,
    pub precision: Option<f32>,
    pub velocity: Option<f32>,
    pub clamp: bool,
    pub bounce: Option<f32>,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            tension: 170.0,
            friction: 26.0,
            mass: 1.0,
            precision: None,
            velocity: None,
            clamp: false,
            bounce: None,
        }
    }
}

impl SpringConfig {
    pub fn gentle() -> Self {
        Self {
            tension: 120.0,
            friction: 14.0,
            ..Default::default()
        }
    }

    pub fn wobbly() -> Self {
        Self {
            tension: 180.0,
            friction: 12.0,
            ..Default::default()
        }
    }

    pub fn stiff() -> Self {
        Self {
            tension: 210.0,
            friction: 20.0,
            ..Default::default()
        }
    }

    pub fn slow() -> Self {
        Self {
            tension: 280.0,
            friction: 60.0,
            ..Default::default()
        }
    }

    pub fn molasses() -> Self {
        Self {
            tension: 280.0,
            friction: 120.0,
            ..Default::default()
        }
    }
}

struct Spring {
    current_value: f32,
    from_value: f32,
    to_value: f32,
    last_velocity: Option<f32>,
}

// def step_spring(
//     dt_secs: float, val: AnimatedValue, config: SpringConfig = SpringConfig()
// ):
fn update_spring(s: &mut Spring, config: SpringConfig) {
    let mut finished = false;
    let v0 = config.velocity.unwrap_or(0.0);
    let precision = config.precision.unwrap_or_else(|| {
        if s.from_value == s.to_value {
            0.005
        } else {
            1.0_f32.min((s.to_value - s.from_value).abs() * 0.001)
        }
    });
    let mut velocity = s.last_velocity.unwrap_or(v0);

    // The velocity at which movement is essentially none
    let rest_velocity = precision / 10.0;

    // Bouncing is opt-in (not to be confused with overshooting)
    let bounce_factor = if config.clamp {
        0.0
    } else {
        config.bounce.unwrap_or(0.0)
    };
    let can_bounce = bounce_factor != 0.0;

    // When `true`, the value is increasing over time
    let is_growing = if s.from_value == s.to_value {
        v0 > 0.0
    } else {
        s.from_value < s.to_value
    };

    // When `true`, the velocity is considered moving
    let mut is_moving: bool;

    // When `true`, the velocity is being deflected or clamped
    let mut is_bouncing;

    let step = 1; // 1ms
    let dt_secs = 1.4214f32;
    let num_steps = (dt_secs / (step as f32 / 1000.0)).ceil() as u32;
    for _ in 0..num_steps {
        is_moving = velocity.abs() > rest_velocity;

        if !is_moving {
            finished = (s.to_value - s.current_value).abs() <= precision;
            if finished {
                break;
            }
        }

        if can_bounce {
            is_bouncing =
                s.current_value == s.to_value || (s.current_value > s.to_value) == is_growing;

            // Invert the velocity with a magnitude, or clamp it.
            if is_bouncing {
                velocity = -velocity * bounce_factor;
                s.current_value = s.to_value;
            }
        }
        let spring_force = -config.tension * 0.000001 * (s.current_value - s.to_value);
        let damping_force = -config.friction * 0.001 * velocity;
        let acceleration = (spring_force + damping_force) / config.mass; // pt/ms^2

        velocity = velocity + acceleration * step as f32; // pt/ms
        s.current_value = s.current_value + velocity * step as f32;
    }
    s.last_velocity = Some(velocity);
    // return finished
}
