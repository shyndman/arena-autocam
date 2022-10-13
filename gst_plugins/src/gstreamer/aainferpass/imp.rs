use crate::detector_service::detect_objects_in_frame;
use crate::CAT;
use gst::{glib, gst_debug as debug, subclass::prelude::*, FractionRange};
use gst_base::subclass::{prelude::*, BaseTransformMode};
use gst_video::subclass::prelude::*;
use gst_video::VideoFrameRef;
use once_cell::sync::Lazy;

// Struct containing all the element data
#[derive(Default)]
pub struct AaInferPass;

// This trait registers our type with the GObject object system and
// provides the entry points for creating a new instance and setting
// up the class data
#[glib::object_subclass]
impl ObjectSubclass for AaInferPass {
    const NAME: &'static str = "AaInferPass";
    type Type = super::AaInferPass;
    type ParentType = gst_video::VideoFilter;
}

// Implementation of glib::Object virtual methods
impl ObjectImpl for AaInferPass {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }
}

impl GstObjectImpl for AaInferPass {}

// Implementation of gst::Element virtual methods
impl ElementImpl for AaInferPass {
    // Set the element specific metadata. This information is what
    // is visible from gst-inspect-1.0 and can also be programmatically
    // retrieved from the gst::Registry after initial registration
    // without having to load the plugin in memory.
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "AaInferPass",
                "VideoFilter",
                "Performs inference as video data passes through",
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
                    FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
                .any_features()
                .build();

            // The sink pad template must be named "sink" for basetransform
            // and specific a pad that is always there
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            // The sink pad template must be named "sink" for basetransform
            // and specific a pad that is always there
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

// Implementation of gst_base::BaseTransform virtual methods
impl BaseTransformImpl for AaInferPass {
    const MODE: gst_base::subclass::BaseTransformMode = BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    // Called for converting caps from one pad to another to account for any
    // changes in the media format this element is performing.
    //
    // In our case that means that:
    fn transform_caps(
        &self,
        _element: &Self::Type,
        _direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_side_caps = caps.clone();

        if let Some(filter) = filter {
            Some(filter.intersect_with_mode(&other_side_caps, gst::CapsIntersectMode::First))
        } else {
            Some(other_side_caps)
        }
    }
}

impl VideoFilterImpl for AaInferPass {
    fn transform_frame_ip(
        &self,
        element: &Self::Type,
        frame_ref: &mut VideoFrameRef<&mut gst::BufferRef>,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        debug!(
            CAT,
            obj: element,
            "Received frame {:#?}",
            frame_ref.buffer().dts_or_pts()
        );
        // let now = std::time::Instant::now();

        detect_objects_in_frame(frame_ref.info().clone(), frame_ref.buffer().copy());

        // let wrapper = FrameBufferWrapper { frame_ref };

        // let (_channels, width, height) = samples.bounds();
        // info!(
        //     CAT,
        //     obj: element,
        //     "Frame received {} {}⨉{}",
        //     frame.buffer().dts_or_pts().unwrap().mseconds(),
        //     width,
        //     height
        // );

        // info!(
        //     CAT,
        //     obj: element,
        //     "Spent {}ms on inference",
        //     now.elapsed().as_millis()
        // );

        Ok(gst::FlowSuccess::Ok)
    }
}
