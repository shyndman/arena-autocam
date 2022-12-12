use std::collections::VecDeque;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use anyhow::Error;
use glib::ObjectExt;
use gst::ClockTime;
use once_cell::sync::Lazy;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_INFER_OVERLAY",
        gst::DebugColorFlags::empty(),
        Some("Auto-Arena Inference"),
    )
});

use crate::config::Config;
use crate::logging::*;
use crate::message::{AAMessage, DetectionDetails};

struct State {
    info: Option<gst_video::VideoInfo>,
    detections: VecDeque<DetectionDetails>,
}

pub fn build_detection_overlay(
    name: &str,
    bus: &gst::Bus,
    config: &Config,
) -> Result<gst::Element, Error> {
    let detection_lifetime_ms: f64 = config
        .detection
        .inference_frame_duration()
        .num_milliseconds() as f64;

    let overlay = gst::ElementFactory::make("cairooverlay")
        .name(name)
        .build()?;
    debug!(CAT, obj: &overlay, "Detection overlay created");

    let state = Arc::new(Mutex::new(State {
        info: None,
        detections: VecDeque::new(),
    }));
    let state_clone = state.clone();

    bus.connect("message", true, move |args| {
        log!(CAT, "Received message signal");

        use gst::MessageView;
        let _bus = args[0].get::<gst::Bus>().unwrap();
        let msg = args[1].get::<gst::Message>().unwrap();

        let app_msg = if let MessageView::Application(app) = msg.view() {
            app
        } else {
            trace!(CAT, "Not an application message");
            return None;
        };

        let app_msg_structure = if let Some(structure) = app_msg.structure() {
            structure
        } else {
            trace!(CAT, "Message did not contain structure");
            return None;
        };

        let app_msg =
            if let Ok(msg) = AAMessage::from_gst_message_structure(app_msg_structure) {
                msg
            } else {
                return None;
            };

        if let AAMessage::InferObjectDetection(details) = app_msg {
            let guard = &mut state_clone.lock().unwrap();
            let detections = &mut guard.detections;
            detections.push_back(details);
        }

        None
    });

    let state_clone = state.clone();
    overlay.connect("draw", true, move |args| {
        let _overlay = args[0].get::<gst::Element>().unwrap();
        let ctx = args[1].get::<cairo::Context>().unwrap();
        let ts = args[2].get::<gst::ClockTime>().unwrap();
        let _dur = args[3].get::<gst::ClockTime>().unwrap();
        let state_guard = &mut state_clone.lock().unwrap();

        debug!(CAT, "Starting overlay frame {:?}", ts);

        let (w, h) = {
            let info = state_guard.info.as_ref().unwrap();
            (info.width() as f64, info.height() as f64)
        };

        let detections = &mut state_guard.detections;
        if detections.is_empty() {
            return None;
        }

        let life_elapsed = |frame_ts: ClockTime, detect_ts: ClockTime| -> f64 {
            let f_ts = frame_ts.mseconds() as f64;
            let d_ts = detect_ts.mseconds() as f64;
            (f_ts - d_ts) / detection_lifetime_ms
        };

        // Expire old detections
        let mut delete_count = 0u16;
        while !detections.is_empty() {
            let life_elapsed_fraction = life_elapsed(ts, detections.front().unwrap().pts);
            if life_elapsed_fraction >= 1.0 {
                debug!(CAT, "life: {}", life_elapsed_fraction);
                delete_count += 1;
                detections.pop_front();
            } else {
                break;
            }
        }
        log!(CAT, "Removed {} detections", delete_count);

        if detections.is_empty() {
            return None;
        }
        ctx.save().expect("Could not save Cairo state");

        log!(CAT, "Drawing {} detections", detections.len());
        for detection in detections.iter() {
            let life_left = 1.0 - life_elapsed(ts, detection.pts);
            if life_left > 1.0 {
                continue;
            }

            let rect = &detection.bounds;

            ctx.set_source_rgba(1.0, 0.0, 0.0, (0.0..1.0).lerp(life_left));
            ctx.set_line_width(1.5);
            ctx.rectangle(
                rect.x() * w,
                rect.y() * h,
                rect.width() * w,
                rect.height() * h,
            );
            ctx.stroke().expect("Failed to draw rect");

            ctx.set_source_rgba(1.0, 0.0, 0.0, (0.0..0.3).lerp(life_left));
            ctx.rectangle(
                rect.x() * w,
                rect.y() * h,
                rect.width() * w,
                rect.height() * h,
            );
            ctx.fill().expect("Failed to fill");
        }

        ctx.restore().expect("Could not restore Cairo state");

        None
    });

    overlay.connect("caps-changed", false, move |args| {
        let _overlay = args[0].get::<gst::Element>().unwrap();
        let caps = args[1].get::<gst::Caps>().unwrap();

        let mut state_guard = state.lock().unwrap();
        state_guard.info = Some(gst_video::VideoInfo::from_caps(&caps).unwrap());

        None
    });

    return Ok(overlay);
}

trait Tweenable {
    fn lerp(&self, t: f64) -> f64;
}

impl Tweenable for Range<f64> {
    fn lerp(&self, t: f64) -> f64 {
        (self.end - self.start) * t + self.start
    }
}
