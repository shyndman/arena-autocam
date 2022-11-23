use aa_foundation::prelude::*;
use aa_hal::{
    clock::timer::Timer,
    motion::spring::{Pulse, Spring, SpringConfig},
    stepper::{get_stepper_pins, FloatDelayToTicks, StepperPins},
    thread::set_thread_as_realtime,
};
use anyhow::{anyhow, Result};
use fugit::{MicrosDurationU32, MicrosDurationU64};
use fugit_timer::Timer as TimerTrait;
use num_format::{CustomFormat, ToFormattedString};
use rand::prelude::*;
use stepper::{
    drivers::a4988::A4988,
    ramp_maker,
    step_mode::{StepMode, StepMode16},
    Direction, Stepper,
};

const RATE_1MHZ: u32 = 1_000_000;
type A4998StepMode = StepMode16;

fn main() -> Result<()> {
    aa_foundation::tracing::setup_dev_tracing_subscriber();
    set_thread_as_realtime();

    let StepperPins {
        ms1_pin,
        ms2_pin,
        ms3_pin,
        reset_pin,
        sleep_pin: _,
        step_pin,
        direction_pin,
    } = get_stepper_pins()?;

    let mut timer = Timer::<RATE_1MHZ>::new_blocking();
    let mut stepper = Stepper::from_driver(A4988::new())
        .enable_direction_control(direction_pin, Direction::Forward, &mut timer)
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_mode_control(
            (reset_pin, ms1_pin, ms2_pin, ms3_pin),
            A4998StepMode::Full,
            &mut timer,
        )
        .map_err(|e| anyhow!("{:?}", e))?
        .enable_step_control(step_pin)
        .enable_motion_control((
            timer.clone(),
            ramp_maker::Flat::<f32>::new(),
            FloatDelayToTicks,
        ));

    let mut spring = Spring::<RATE_1MHZ>::new(0.0, 1200.0, 0.001);
    let sping_config = SpringConfig {
        clamp: false,
        tension: 200.0,
        friction: 4.0,
        mass: 12.0,
        ..SpringConfig::default()
    };

    let mut frame = 0;
    let mut retargets = 0;
    let mut acc_delay = MicrosDurationU64::from_ticks(0);
    let mut pulse_update: Option<(f64, MicrosDurationU64)> = None;
    let formatter = CustomFormat::builder().separator(",").build()?;

    loop {
        if let Some(Pulse {
            mut delay,
            current_value,
            mut next_value,
            velocity,
            acceleration,
        }) = spring.next_pulse(pulse_update, &sping_config)
        {
            info!(
                indoc! {"
                    step #{}
                         val: {:.4}
                        Δval: {:+.8}
                         acc: {:+.8}
                        wait: {}µs (total: {}ms)"},
                frame,
                next_value,
                velocity,
                acceleration,
                delay.to_micros().to_formatted_string(&formatter),
                (acc_delay + delay)
                    .to_millis()
                    .to_formatted_string(&formatter),
            );

            let step = find_ideal_microstep(delay);
            if step != A4998StepMode::Full {
                let denom: u16 = step.into();
                next_value = current_value + (velocity.signum() * 1.0 / denom as f64);
                delay = delay / (denom as u32);
                info!(
                    indoc! {"
                        configuring for 1/{}th microstep to smooth out a long delay
                             val: {:.4}
                            wait: {}µs ({}ms)
                    "},
                    denom,
                    next_value,
                    delay.to_micros().to_formatted_string(&formatter),
                    (acc_delay + delay)
                        .to_millis()
                        .to_formatted_string(&formatter),
                );
            }

            // TODO: We should be keeping track of what step mode we're in to avoid
            // introducing unnecessary setup delays
            let mut f = stepper.set_step_mode(step, &mut timer);
            f.wait().map_err(|e| anyhow!("{:?}", e))?;
            f.release();

            stepper
                .set_direction(
                    if next_value > current_value {
                        Direction::Forward
                    } else {
                        Direction::Backward
                    },
                    &mut timer,
                )
                .wait()
                .map_err(|e| anyhow!("{:?}", e))?;

            let mut timer = timer.clone();
            timer.start(MicrosDurationU32::from_ticks(delay.ticks() as u32))?;
            loop {
                match timer.wait() {
                    Ok(_) => break,
                    Err(e) => match e {
                        nb::Error::Other(e) => Err(anyhow!("{:?}", e))?,
                        nb::Error::WouldBlock => continue,
                    },
                }
            }
            stepper
                .step(&mut timer)
                .wait()
                .map_err(|e| anyhow!("{:?}", e))?;

            acc_delay += delay;
            pulse_update = Some((next_value, delay));
        } else if retargets < 10 {
            info!("done");
            frame = 0;
            retargets += 1;
            acc_delay = MicrosDurationU64::from_ticks(0);
            spring.update_target_by(thread_rng().gen_range(-5..=5) as f64 * 100.0);
        } else {
            break;
        }

        frame += 1;
    }

    Ok(())
}

/// The delay under
const DELAY_THRESHOLD_FOR_MICROSTEP_US: MicrosDurationU64 =
    MicrosDurationU64::from_ticks(20_000);

pub fn find_ideal_microstep(delay: MicrosDurationU64) -> A4998StepMode {
    if delay > DELAY_THRESHOLD_FOR_MICROSTEP_US {
        A4998StepMode::iter()
            .skip(1) // Skip Full
            .find(|step| {
                let step_denom: u16 = (*step).into();
                let delay_with_microstep = delay / (step_denom as u32);

                // If we're below the threshold while at this step, then we've found what
                // we're looking for
                delay_with_microstep < DELAY_THRESHOLD_FOR_MICROSTEP_US
            })
            .unwrap_or(A4998StepMode::M16)
    } else {
        A4998StepMode::Full
    }
}
