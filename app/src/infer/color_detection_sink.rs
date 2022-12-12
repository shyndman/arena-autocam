use once_cell::sync::Lazy;

pub(self) static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_COLOR_DETECT_SINK",
        gst::DebugColorFlags::FG_YELLOW,
        Some("Auto-Arena Inference"),
    )
});

glib::wrapper! {
    pub struct ColorDetectionSink(ObjectSubclass<imp::ColorDetectionSink>) @extends gst_video::VideoSink, gst_base::BaseSink, gst::Element, gst::Object;
}

/// API visible to consumers
impl ColorDetectionSink {
    pub fn new(name: Option<&str>) -> Self {
        glib::Object::new(&[("name", &name), ("qos", &true)])
    }
}

mod imp {
    use std::sync::Mutex;
    use std::time::Instant;

    use aa_foundation::image::find_similar_color_regions;
    // use aa_foundation::image::find_similar_color_regions;
    use anyhow::Result;
    use glib::{ParamSpecBuilderExt, ToValue};
    use gst::subclass::prelude::*;
    use gst::{glib, FlowError, Fraction};
    use gst_base::subclass::prelude::*;
    use gst_video::subclass::prelude::*;
    use gst_video::{VideoCapsBuilder, VideoFormat, VideoFrameRef, VideoInfo};
    use image::Rgb;
    use once_cell::sync::Lazy;
    use palette::Lab;

    use super::CAT;
    use crate::foundation::geom::Rect;
    use crate::logging::*;
    use crate::message::{AAMessage, DetectionDetails};

    const TARGET_COLOR_LIGHT: Lab = Lab::new(61.1959, -36.404343, 11.648214);
    const TARGET_COLOR_DARK: Lab = Lab::new(40.29259, -32.861248, 11.006456);
    const TARGET_COLORS: [&Lab; 2] = [&TARGET_COLOR_LIGHT, &TARGET_COLOR_DARK];

    #[derive(Default)]
    struct PropsStorage {
        max_results: u32,
        detection_pixel_threshold: u32,
        bus: Option<gst::Bus>,
    }

    // Struct containing all the element data
    #[derive(Default)]
    pub struct ColorDetectionSink {
        props_storage: Mutex<PropsStorage>,
        video_info: Mutex<Option<VideoInfo>>,
    }

    impl ColorDetectionSink {
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

        fn perform_detection(
            &self,
            buffer: &gst::Buffer,
        ) -> Result<Vec<DetectionDetails>, gst::FlowError> {
            let detection_pixel_threshold = {
                let guard = self.props_storage.lock().unwrap();
                guard.detection_pixel_threshold
            };
            let mut info_guard = self.video_info.lock().unwrap();
            let video_info = info_guard.as_mut().ok_or_else(|| {
                gst::element_error!(
                    &self.obj(),
                    gst::CoreError::Negotiation,
                    ["Have no info yet"]
                );
                gst::FlowError::NotNegotiated
            })?;

            let readable = buffer.map_readable().unwrap();
            let frame =
                VideoFrameRef::from_buffer_ref_readable(readable.buffer(), video_info)
                    .unwrap();
            let w = frame.width();
            let h = frame.height();
            let flat_samples = image::FlatSamples::<Vec<u8>> {
                samples: frame.plane_data(0).unwrap().into(),
                layout: image::flat::SampleLayout {
                    channels: 3,       // RGB
                    channel_stride: 1, // 1 byte from component to component
                    width: frame.width(),
                    width_stride: 3, // 3 byte from pixel to pixel
                    height: frame.height(),
                    height_stride: frame.plane_stride()[0] as usize, // stride from line to line
                },
                color_hint: Some(image::ColorType::Rgb8),
            };
            let image = match flat_samples.try_into_buffer::<Rgb<u8>>() {
                Ok(i) => i,
                Err(e) => {
                    error!(CAT, "Error: {:?}", e);
                    panic!();
                }
            };

            let regions = find_similar_color_regions(&image, &TARGET_COLORS).unwrap();
            Ok(regions
                .iter()
                .filter(|r| r.count > detection_pixel_threshold)
                .map(|r| {
                    let fractional_bounds = Rect::new(
                        r.x() as f64 / w as f64,
                        r.y() as f64 / h as f64,
                        r.width() as f64 / w as f64,
                        r.height() as f64 / h as f64,
                    );

                    DetectionDetails {
                        label: "Color".into(),
                        pts: buffer.pts().unwrap(),
                        bounds: fractional_bounds,
                        ..Default::default()
                    }
                })
                .collect())
        }
    }

    // This trait registers our type with the GObject object system and
    // provides the entry points for creating a new instance and setting
    // up the class data
    #[glib::object_subclass]
    impl ObjectSubclass for ColorDetectionSink {
        const NAME: &'static str = "ColorDetectionSink";
        type Type = super::ColorDetectionSink;
        type ParentType = gst_video::VideoSink;
    }

    // Implementation of glib::Object virtual methods
    impl ObjectImpl for ColorDetectionSink {
        fn signals() -> &'static [glib::subclass::Signal] {
            &[]
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecUInt::builder("max-results")
                        .nick("Max result count")
                        .blurb("The maximum number of detections to return per frame")
                        .default_value(3)
                        .build(),
                    glib::ParamSpecUInt::builder("detection-pixel-threshold")
                        .nick("Detection pixel threshold")
                        .blurb("The minimum number of pixels in the target color that is considered a detection")
                        .default_value(10)
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
                "max-results" => {
                    props_guard.max_results =
                        value.get::<u32>().expect("type checked upstream");
                }
                "detection-pixel-threshold" => {
                    props_guard.detection_pixel_threshold =
                        value.get::<u32>().expect("type checked upstream")
                }
                "bus" => props_guard.bus = value.get().expect("type checked upstream"),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let props_guard = self.props_storage.lock().unwrap();
            match pspec.name() {
                "max-results" => props_guard.max_results.to_value(),
                "detection-pixel-threshold" => {
                    props_guard.detection_pixel_threshold.to_value()
                }
                "bus" => props_guard.bus.to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl GstObjectImpl for ColorDetectionSink {}

    impl ElementImpl for ColorDetectionSink {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "ColorDetectionSink",
                    "Sink",
                    "Performs color detection for debugging.",
                    "Scott Hyndman <shyndman@gmail.com>",
                )
            });

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gst::PadTemplate] {
            static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
                let caps = VideoCapsBuilder::new()
                    .format(VideoFormat::Rgb)
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

    impl BaseSinkImpl for ColorDetectionSink {
        fn set_caps(&self, caps: &gst::Caps) -> Result<(), gst::LoggableError> {
            let instance = self.obj();
            info!(CAT, obj: instance, "Setting caps to {:?}", caps);

            let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();
            let mut info_guard = self.video_info.lock().unwrap();
            info_guard.replace(video_info);

            Ok(())
        }
    }

    impl VideoSinkImpl for ColorDetectionSink {
        fn show_frame(
            &self,
            buffer: &gst::Buffer,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {
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
            if res.len() != 0 {
                debug!(
                    CAT,
                    "Detected {} object{}",
                    res.len(),
                    if res.len() == 1 { "" } else { "s" }
                );
            }
            for d in res.iter() {
                self.post_to_bus(
                    AAMessage::InferObjectDetection(d.clone())
                        .to_gst_message()
                        .map_err(|_| gst::FlowError::Error)?,
                )
                .map_err(|_| FlowError::Error)?;
            }

            // Signal that the frame is now complete
            self.post_to_bus(
                AAMessage::InferFrameDone {
                    dts: dts,
                    detection_count: res.len() as i32,
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
