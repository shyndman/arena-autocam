// Required to call the `ramp` method.
use ramp_maker::MotionProfile;

fn main() {
    // Let's use floating point numbers here to keep the example simple.
    // RampMaker also supports fixed-point numbers though.
    let target_accel = 200.0; // meters per second^2
    let max_velocity = 1500.0; // meters per second

    let mut profile = ramp_maker::Trapezoidal::new(target_accel);

    let num_steps = 2000;
    profile.enter_position_mode(max_velocity, num_steps);
    let velocities: Vec<f32> = profile.velocities().collect();

    profile.enter_position_mode(max_velocity, num_steps);
    let accelerations: Vec<f32> = profile.accelerations().collect();

    profile.enter_position_mode(max_velocity, num_steps);
    let delays: Vec<f32> = profile.delays().collect();

    for i in 0..delays.len() {
        let delay = delays[i];
        let vel = velocities[i];
        let accel = if i + 1 == delays.len() {
            0.0
        } else {
            accelerations[i]
        };

        eprintln!(
            "velocity: {}, acceleration: {}\ndelay by {}",
            vel, accel, delay,
        );
    }
}
