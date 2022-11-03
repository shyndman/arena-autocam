use std::{
    collections::VecDeque,
    ops::Range,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use glib::ObjectExt;
use gst::ClockTime;

use crate::{
    infer::CAT,
    logging::*,
    message::{AAMessage, DetectionDetails},
};

struct State {
    info: Option<gst_video::VideoInfo>,
    detections: VecDeque<DetectionDetails>,
}

const DETECTION_LIFETIME_MS: u64 = 2000;

pub fn build_detection_overlay(name: &str, bus: &gst::Bus) -> Result<gst::Element, Error> {
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
        trace!(CAT, "Received message signal");

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

    // bus.add_watch(move |_bus, msg| {
    // })?;

    let state_clone = state.clone();
    overlay.connect("draw", true, move |args| {
        let _overlay = args[0].get::<gst::Element>().unwrap();
        let ctx = args[1].get::<cairo::Context>().unwrap();
        let ts = args[2].get::<gst::ClockTime>().unwrap();
        let _dur = args[3].get::<gst::ClockTime>().unwrap();
        let state_guard = &mut state_clone.lock().unwrap();

        let (w, h) = {
            let info = state_guard.info.as_ref().unwrap();
            (info.width() as f64, info.height() as f64)
        };

        let detections = &mut state_guard.detections;
        if detections.is_empty() {
            return None;
        }

        fn life_elapsed(frame_ts: ClockTime, detect_ts: ClockTime) -> f64 {
            (frame_ts - detect_ts).mseconds() as f64 / DETECTION_LIFETIME_MS as f64
        }

        // Expire old detections
        while !detections.is_empty() {
            let life_elapsed_fraction = life_elapsed(ts, detections.front().unwrap().dts);
            if life_elapsed_fraction >= 1.0 {
                detections.pop_front();
            } else {
                break;
            }
        }

        ctx.save().expect("Could not save Cairo state");

        for detection in detections.iter() {
            let life_left = 1.0 - life_elapsed(ts, detection.dts);
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
