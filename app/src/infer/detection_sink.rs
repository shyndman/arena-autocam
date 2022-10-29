use super::CAT;

glib::wrapper! {
    pub struct DetectionSink(ObjectSubclass<imp::DetectionSink>) @extends gst_video::VideoSink, gst_base::BaseSink, gst::Element, gst::Object;
}

/// API visible to consumers
impl DetectionSink {
    pub fn new(name: Option<&str>) -> Self {
        glib::Object::new(&[("name", &name)])
    }
}

mod imp {
    use std::sync::Mutex;

    use anyhow::Result;
    use glib::{ParamSpecBuilderExt, ToValue};
    use gst::{debug, glib, info, subclass::prelude::*, FlowError, Fraction};
    use gst_base::subclass::prelude::*;
    use gst_video::{subclass::prelude::*, VideoCapsBuilder, VideoFormat, VideoInfo};
    use once_cell::sync::Lazy;
    use rand::Rng;

    use super::CAT;
    use crate::{
        message::{DetectionFrameDone, DetectionFrameStart, DetectionMsg, ObjectDetection},
        util::random_fraction_rect,
    };

    #[derive(Default)]
    struct Settings {
        model_location: Option<String>,
        max_results: u32,
        score_threshold: f32,
        bus: Option<gst::Bus>,
    }

    // Struct containing all the element data
    #[derive(Default)]
    pub struct DetectionSink {
        info: Mutex<Option<VideoInfo>>,
        settings: Mutex<Settings>,
    }

    impl DetectionSink {
        fn post_to_bus(&self, msg: gst::Message) -> Result<()> {
            debug!(CAT, obj: self.instance(), "Posting message on bus, {:?}", msg);

            let settings = self.settings.lock().unwrap();
            if let Some(bus) = settings.bus.as_ref() {
                bus.post(msg)?;
                Ok(())
            } else {
                Err(anyhow::Error::msg("No bus assigned"))
            }
        }
    }

    // This trait registers our type with the GObject object system and
    // provides the entry points for creating a new instance and setting
    // up the class data
    #[glib::object_subclass]
    impl ObjectSubclass for DetectionSink {
        const NAME: &'static str = "DetectionSink";
        type Type = super::DetectionSink;
        type ParentType = gst_video::VideoSink;
    }

    // Implementation of glib::Object virtual methods
    impl ObjectImpl for DetectionSink {
        fn signals() -> &'static [glib::subclass::Signal] {
            &[]
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("model-location")
                        .nick("Model location")
                        .blurb("Path to the .tflite file")
                        .build(),
                    glib::ParamSpecUInt::builder("max-results")
                        .nick("Max result count")
                        .blurb("The maximum number of detections to return per frame")
                        .default_value(3)
                        .build(),
                    glib::ParamSpecFloat::builder("score-threshold")
                        .nick("Minimum score threshold")
                        .blurb("The minimum score a detection must meet to be returned")
                        .default_value(0.3)
                        .build(),
                    glib::ParamSpecObject::builder::<gst::Bus>("bus")
                        .nick("Pipeline bus")
                        .blurb("The bus to which detection messages are written")
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let mut settings = self.settings.lock().unwrap();
            match pspec.name() {
                "model-location" => {
                    settings.model_location = value.get().expect("type checked upstream")
                }
                "max-results" => {
                    settings.max_results = value.get::<u32>().expect("type checked upstream")
                }
                "score-threshold" => {
                    settings.score_threshold = value.get().expect("type checked upstream")
                }
                "bus" => settings.bus = value.get().expect("type checked upstream"),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let settings = self.settings.lock().unwrap();
            match pspec.name() {
                "model-location" => settings.model_location.to_value(),
                "max-results" => settings.max_results.to_value(),
                "score-threshold" => settings.score_threshold.to_value(),
                "bus" => settings.bus.to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl GstObjectImpl for DetectionSink {}

    impl ElementImpl for DetectionSink {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "DetectionSink",
                    "Sink",
                    "Performs Arena Autocam detection as a sink.",
                    "Scott Hyndman <shyndman@gmail.com>",
                )
            });

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gst::PadTemplate] {
            static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
                let caps = VideoCapsBuilder::new()
                    .format(VideoFormat::I420)
                    .width_range(0..=i32::MAX)
                    .height_range(0..=i32::MAX)
                    .framerate_range(Fraction::new(0, 1)..=Fraction::new(i32::MAX, 1))
                    .build();

                // The sink pad template must be named "sink" for basetransform
                // and specific a pad that is always there
                let sink_pad_template = gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();

                vec![sink_pad_template]
            });

            PAD_TEMPLATES.as_ref()
        }
    }

    impl BaseSinkImpl for DetectionSink {
        fn set_caps(&self, caps: &gst::Caps) -> Result<(), gst::LoggableError> {
            info!(CAT, obj: self.obj(), "Setting caps to {:?}", caps);

            let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();
            let mut info = self.info.lock().unwrap();
            info.replace(video_info);

            Ok(())
        }
    }

    impl VideoSinkImpl for DetectionSink {
        fn show_frame(
            &self,
            buffer: &gst::Buffer,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {
            let mut info_guard = self.info.lock().unwrap();
            let _info = info_guard.as_mut().ok_or_else(|| {
                gst::element_error!(
                    &self.obj(),
                    gst::CoreError::Negotiation,
                    ["Have no info yet"]
                );
                gst::FlowError::NotNegotiated
            })?;

            let dts = buffer.dts_or_pts().unwrap();
            debug!(CAT, obj: self.obj(), "Showing frame {}", dts);

            self.post_to_bus(
                gst::message::Application::builder(
                    DetectionFrameStart { dts }.to_structure(),
                )
                .build(),
            )
            .map_err(|_| FlowError::Error)?;

            std::thread::sleep(std::time::Duration::from_millis(1000 / 3));

            let mut r = rand::thread_rng();
            if r.gen_range(0.0..=1.0) > 0.4 {
                let score = 0.44f32;
                let bounds = random_fraction_rect();
                let label = String::from("horse");
                self.post_to_bus(
                    gst::message::Application::builder(
                        ObjectDetection {
                            dts,
                            label,
                            score,
                            bounds,
                        }
                        .to_structure(),
                    )
                    .build(),
                )
                .map_err(|_| FlowError::Error)?;
            }

            let objects_detected = 3;
            self.post_to_bus(
                gst::message::Application::builder(
                    DetectionFrameDone {
                        dts,
                        detection_count: objects_detected,
                    }
                    .to_structure(),
                )
                .build(),
            )
            .map_err(|_| FlowError::Error)?;

            Ok(gst::FlowSuccess::Ok)
        }
    }
}
