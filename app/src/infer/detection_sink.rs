use once_cell::sync::Lazy;

pub(self) static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_INFER_SINK",
        gst::DebugColorFlags::FG_YELLOW,
        Some("Auto-Arena Inference"),
    )
});

pub(self) static DETECT_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_DETECT",
        gst::DebugColorFlags::FG_RED,
        Some("Auto-Arena Detection"),
    )
});

glib::wrapper! {
    pub struct DetectionSink(ObjectSubclass<imp::DetectionSink>) @extends gst_video::VideoSink, gst_base::BaseSink, gst::Element, gst::Object;
}

/// API visible to consumers
impl DetectionSink {
    pub fn new(name: Option<&str>) -> Self {
        glib::Object::new(&[("name", &name), ("qos", &true)])
    }
}

mod imp {
    use std::{sync::Mutex, time::Instant};

    use anyhow::Result;
    use glib::{ParamSpecBuilderExt, ToValue};
    use gst::{glib, subclass::prelude::*, FlowError, Fraction};
    use gst_base::subclass::prelude::*;
    use gst_video::{subclass::prelude::*, VideoCapsBuilder, VideoFormat, VideoInfo};
    use once_cell::sync::Lazy;
    use tflite_support::{BaseOptions, DetectionOptions, DetectionResult, ObjectDetector};

    use super::{CAT, DETECT_CAT};
    use crate::logging::*;
    use crate::{
        foundation::geom::Rect,
        infer::tf_buffer_adapter::TensorflowBufferAdapter,
        message::{AAMessage, DetectionDetails},
    };

    #[derive(Default)]
    struct PropsStorage {
        model_location: Option<String>,
        max_results: u32,
        score_threshold: f32,
        bus: Option<gst::Bus>,
        model_invalidated: bool,
    }

    // Struct containing all the element data
    #[derive(Default)]
    pub struct DetectionSink {
        props_storage: Mutex<PropsStorage>,
        video_info: Mutex<Option<VideoInfo>>,
        object_detector: Mutex<Option<ObjectDetector>>,
    }

    impl DetectionSink {
        fn get_dimensions(&self) -> Result<(f64, f64)> {
            let info_guard = self.video_info.lock().unwrap();
            let info = info_guard.as_ref().unwrap();
            Ok((info.width() as f64, info.height() as f64))
        }

        fn post_to_bus(&self, msg: gst::Message) -> Result<()> {
            let instance = self.instance();
            log!(CAT, obj: &instance, "Posting message on bus, {:?}", msg);

            let props_guard = self.props_storage.lock().unwrap();
            if let Some(bus) = props_guard.bus.as_ref() {
                bus.post(msg)?;
                Ok(())
            } else {
                Err(anyhow::Error::msg("No bus assigned"))
            }
        }

        fn is_model_invalidated(&self) -> bool {
            let props_guard = self.props_storage.lock().unwrap();
            props_guard.model_invalidated && props_guard.model_location.is_some()
        }

        fn create_inference_model(&self) -> Result<()> {
            let mut props_guard = self.props_storage.lock().unwrap();
            let model_location = props_guard.model_location.as_ref().unwrap();
            info!(
                CAT,
                "Creating new inference model, path={}", &model_location
            );

            let object_detector = ObjectDetector::with_options(
                BaseOptions {
                    model_path: model_location.clone(),
                    ..Default::default()
                },
                DetectionOptions {
                    max_results: Some(props_guard.max_results as i32),
                    score_threshold: Some(props_guard.score_threshold),
                },
            )?;

            let mut detector_guard = self.object_detector.lock().unwrap();
            *detector_guard = Some(object_detector);
            props_guard.model_invalidated = false;

            Ok(())
        }

        fn perform_detection(
            &self,
            buffer: &gst::Buffer,
        ) -> Result<DetectionResult, gst::FlowError> {
            let mut info_guard = self.video_info.lock().unwrap();
            let video_info = info_guard.as_mut().ok_or_else(|| {
                gst::element_error!(
                    &self.obj(),
                    gst::CoreError::Negotiation,
                    ["Have no info yet"]
                );
                gst::FlowError::NotNegotiated
            })?;

            let detector_guard = self.object_detector.lock().unwrap();
            let detector = detector_guard
                .as_ref()
                .expect("We don't have an object detector, but should");

            Ok(detector
                .detect(TensorflowBufferAdapter { video_info, buffer })
                .map_err(|_| FlowError::Error)?)
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
            let mut props_guard = self.props_storage.lock().unwrap();
            match pspec.name() {
                "model-location" => {
                    props_guard.model_location = value.get().expect("type checked upstream");
                    props_guard.model_invalidated = true
                }
                "max-results" => {
                    props_guard.max_results =
                        value.get::<u32>().expect("type checked upstream");
                    props_guard.model_invalidated = true
                }
                "score-threshold" => {
                    props_guard.score_threshold = value.get().expect("type checked upstream");
                    props_guard.model_invalidated = true
                }
                "bus" => props_guard.bus = value.get().expect("type checked upstream"),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let props_guard = self.props_storage.lock().unwrap();
            match pspec.name() {
                "model-location" => props_guard.model_location.to_value(),
                "max-results" => props_guard.max_results.to_value(),
                "score-threshold" => props_guard.score_threshold.to_value(),
                "bus" => props_guard.bus.to_value(),
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
            let instance = self.obj();
            info!(CAT, obj: instance, "Setting caps to {:?}", caps);

            let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();
            let mut info_guard = self.video_info.lock().unwrap();
            info_guard.replace(video_info);

            Ok(())
        }
    }

    impl VideoSinkImpl for DetectionSink {
        fn show_frame(
            &self,
            buffer: &gst::Buffer,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {
            // Create the model if it's been invalidated, or is not yet created
            if self.is_model_invalidated() {
                self.create_inference_model()
                    .map_err(|_| FlowError::Error)?;
            }

            let start_ts = Instant::now();

            // Notify that we're beginning the inference frame
            let dts = buffer.pts().unwrap();
            debug!(CAT, "Starting inference frame {:?}", dts,);
            self.post_to_bus(
                AAMessage::InferFrameStart { dts: dts }
                    .to_gst_message()
                    // TODO(shyndman): We really need to report the error text
                    .map_err(|_| gst::FlowError::Error)?,
            )
            .map_err(|_| FlowError::Error)?;

            // Run object detection on the frame
            let res = self.perform_detection(buffer)?;
            if res.size() != 0 {
                debug!(
                    DETECT_CAT,
                    "Detected {} object{}",
                    res.size(),
                    if res.size() == 1 { "" } else { "s" }
                );
            }
            for d in res.detections() {
                let (label, score) = {
                    let first_category = d
                        .categories()
                        .next()
                        .expect("A detection result should have at least one category");
                    (first_category.label_as_string(), first_category.score())
                };

                debug!(DETECT_CAT, "label={:<8} score={}", label, score);

                let (w, h) = self.get_dimensions().unwrap();
                let object_bounds = d.bounding_box();
                let fractional_bounds = Rect::new(
                    object_bounds.x as f64 / w,
                    object_bounds.y as f64 / h,
                    object_bounds.width as f64 / w,
                    object_bounds.height as f64 / h,
                );

                self.post_to_bus(
                    AAMessage::InferObjectDetection(DetectionDetails {
                        pts: dts,
                        label: label.into(),
                        score,
                        bounds: fractional_bounds,
                    })
                    .to_gst_message()
                    .map_err(|_| gst::FlowError::Error)?,
                )
                .map_err(|_| FlowError::Error)?;
            }

            // Signal that the frame is now complete
            self.post_to_bus(
                AAMessage::InferFrameDone {
                    dts: dts,
                    detection_count: res.size() as i32,
                    duration: start_ts.elapsed(),
                }
                .to_gst_message()
                // TODO(shyndman): We really need to report the error text
                .map_err(|_| gst::FlowError::Error)?,
            )
            .map_err(|_| FlowError::Error)?;

            debug!(
                CAT,
                "Finished inference frame {:?}, duration={}",
                dts,
                start_ts.elapsed().as_secs_f32(),
            );

            Ok(gst::FlowSuccess::Ok)
        }
    }
}
