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
    message::{AppMsgType, Category, DetectionFrameDone, ObjectDetection},
};

struct State {
    info: Option<gst_video::VideoInfo>,
    detections: VecDeque<ObjectDetection>,
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
        use gst::MessageView;

        trace!(CAT, "Received message signal");
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

        let app_msg_type = if let Ok(msg_type) =
            AppMsgType::try_from(app_msg_structure.name().to_string())
        {
            trace!(CAT, "Message has valid AppMsgType, {}", msg_type);
            msg_type
        } else {
            warning!(
                CAT,
                "Failed to read AppMsgType, {}",
                app_msg_structure.name()
            );
            return None;
        };

        if app_msg_type.category != Category::Detection {
            trace!(
                CAT,
                "Message not in the detection category, {}",
                app_msg_type.category
            );
            return None;
        }

        trace!(CAT, "Parsing message with name, {}", app_msg_type.name);
        match app_msg_type.name.as_str() {
            "object-detection" => {
                let msg: ObjectDetection = app_msg_structure.try_into().unwrap();
                trace!(CAT, "Parsed ObjectDetection");

                let state = &mut state_clone.lock().unwrap();
                let detections = &mut state.detections;
                detections.push_back(msg);
            }
            "detection-frame-done" => {
                let _msg: DetectionFrameDone = app_msg_structure.try_into().unwrap();
            }
            _ => {}
        }

        None
    });

    // bus.add_watch(move |_bus, msg| {
    // })?;

    let state_clone = state.clone();
    overlay.connect("draw", true, move |args| {
        let ts = args[2].get::<gst::ClockTime>().unwrap();
        let _overlay = args[0].get::<gst::Element>().unwrap();
        let ctx = args[1].get::<cairo::Context>().unwrap();
        let _dur = args[3].get::<gst::ClockTime>().unwrap();
        let state = &mut state_clone.lock().unwrap();

        debug!(CAT, "Drawing overlay frame, {}", ts);

        let (w, h) = {
            let info = state.info.as_ref().unwrap();
            (info.width() as f64, info.height() as f64)
        };

        fn life_elapsed(frame_ts: ClockTime, detect_ts: ClockTime) -> f64 {
            (frame_ts - detect_ts).mseconds() as f64 / DETECTION_LIFETIME_MS as f64
        }

        // Expire old detections
        let detections = &mut state.detections;
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
            let rect = detection.bounds;

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

        let mut drawer = state.lock().unwrap();
        drawer.info = Some(gst_video::VideoInfo::from_caps(&caps).unwrap());

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

    // fn rem(self, rhs: f64) -> Self::Output {
    //     self.lerp(rhs)
    // }
}
