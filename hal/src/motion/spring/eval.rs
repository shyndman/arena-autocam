#[cfg(test)]
mod test {
    use aa_foundation::{prelude::*, tracing::setup_dev_tracing_subscriber};

    use crate::motion::spring::SpringConfig;
    const TENSION_SCALE: f64 = 0.000001;
    const FRICTION_SCALE: f64 = 0.001;

    #[test]
    fn unit_scratch() {
        setup_dev_tracing_subscriber();

        let config = SpringConfig::default();
        let current_value = 100.0;
        let to_value = 110.0;
        let mut velocity = 0.0;

        let mut spring_force = 0.0;
        let mut damping_force = 0.0;
        let mut acceleration = 0.0;

        for _ in 0..5 {
            spring_force = -config.tension * TENSION_SCALE * (current_value - to_value);
            damping_force = -config.friction * FRICTION_SCALE * velocity;
            acceleration = (spring_force + damping_force) / config.mass; // space_units / time_unit^2
            velocity += acceleration; // space_units / time_unit
        }

        debug!(
            indoc! {"
                \nspring_force={spring_force:.8}
                damping_force={damping_force:.8}
                acceleration={acceleration:.8}
                velocity={velocity:.8}
            "},
            spring_force = spring_force,
            damping_force = damping_force,
            acceleration = acceleration,
            velocity = velocity,
        );
    }
}
