use super::super::CAT;
use gst::glib;
use gst::subclass::prelude::*;
use gst::{gst_debug as debug, gst_info as info};
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;
use gst_video::VideoFrameRef;
use gst_video::VideoInfo;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Duration;

// Struct containing all the element data
#[derive(Default)]
pub struct AaInferSink {
    pub info: Mutex<Option<VideoInfo>>,
}

// This trait registers our type with the GObject object system and
// provides the entry points for creating a new instance and setting
// up the class data
#[glib::object_subclass]
impl ObjectSubclass for AaInferSink {
    const NAME: &'static str = "AaInferSink";
    type Type = super::AaInferSink;
    type ParentType = gst_video::VideoSink;
}

// Implementation of glib::Object virtual methods
impl ObjectImpl for AaInferSink {}

impl GstObjectImpl for AaInferSink {}

// Implementation of gst::Element virtual methods
impl ElementImpl for AaInferSink {
    // Set the element specific metadata. This information is what
    // is visible from gst-inspect-1.0 and can also be programmatically
    // retrieved from the gst::Registry after initial registration
    // without having to load the plugin in memory.
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> =
            Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "AaInferSink",
                    "Sink",
                    "Performs Arena-Autocam inference as a sink.",
                    "Scott Hyndman <shyndman@gmail.com>",
                )
            });

        Some(&*ELEMENT_METADATA)
    }

    // Create and add pad templates for our sink pad. These
    // are later used for actually creating the pads and beforehand
    // already provide information to GStreamer about all possible
    // pads that could exist for this type.
    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst::Caps::builder("video/x-raw")
                .field("format", gst_video::VideoFormat::Rgb.to_str())
                .field("width", gst::IntRange::new(0, i32::MAX))
                .field("height", gst::IntRange::new(0, i32::MAX))
                .field(
                    "framerate",
                    gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
                .any_features()
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

// Implementation of gst_base::BaseTransform virtual methods
impl BaseSinkImpl for AaInferSink {
    fn set_caps(
        &self,
        element: &Self::Type,
        caps: &gst::Caps,
    ) -> Result<(), gst::LoggableError> {
        let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();
        let mut info = self.info.lock().unwrap();
        info.replace(video_info);

        info!(CAT, obj: element, "Setting caps to {:?}", caps);

        Ok(())
    }
}

impl VideoSinkImpl for AaInferSink {
    fn show_frame(
        &self,
        element: &Self::Type,
        buffer: &gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        debug!(CAT, obj: element, "Received frame len={}", buffer.size());
        let now = std::time::Instant::now();

        std::thread::sleep(Duration::from_millis(100));

        let mut info_guard = self.info.lock().unwrap();
        let info = info_guard.as_mut().ok_or_else(|| {
            gst::element_error!(
                element,
                gst::CoreError::Negotiation,
                ["Have no info yet"]
            );
            gst::FlowError::NotNegotiated
        })?;

        let frame =
            VideoFrameRef::from_buffer_ref_readable(buffer.as_ref(), info).unwrap();
        let samples = image::FlatSamples::<Vec<u8>> {
            samples: frame.plane_data(0).unwrap().to_vec(),
            layout: image::flat::SampleLayout {
                channels: 1,
                channel_stride: 1,
                width: frame.width(),
                width_stride: 1,
                height: frame.height(),
                height_stride: frame.plane_stride()[0] as usize,
            },
            color_hint: None,
        };

        let (_channels, width, height) = samples.bounds();
        info!(
            CAT,
            obj: element,
            "Frame received {} {}⨉{}",
            buffer.dts_or_pts().unwrap().mseconds(),
            width,
            height
        );

        info!(
            CAT,
            obj: element,
            "Spent {}ms on inference",
            now.elapsed().as_millis()
        );

        Ok(gst::FlowSuccess::Ok)
    }
}
