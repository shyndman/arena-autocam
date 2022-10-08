use gst::glib;
use gst::gst_debug as debug;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use once_cell::sync::Lazy;

// This module contains the private implementation details of our element

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "aainfer",
        gst::DebugColorFlags::empty(),
        Some("AaInfer Element"),
    )
});

// Struct containing all the element data
#[derive(Default)]
pub struct AaInfer {}

// This trait registers our type with the GObject object system and
// provides the entry points for creating a new instance and setting
// up the class data
#[glib::object_subclass]
impl ObjectSubclass for AaInfer {
    const NAME: &'static str = "AaInfer";
    type Type = super::AaInfer;
    type ParentType = gst_video::VideoFilter;
}

// Implementation of glib::Object virtual methods
impl ObjectImpl for AaInfer {}

impl GstObjectImpl for AaInfer {}

// Implementation of gst::Element virtual methods
impl ElementImpl for AaInfer {
    // Set the element specific metadata. This information is what
    // is visible from gst-inspect-1.0 and can also be programmatically
    // retrieved from the gst::Registry after initial registration
    // without having to load the plugin in memory.
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "AaInfer",
                "Sink",
                "Does nothing with the data",
                "Scott Hyndman <shyndman@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    // Create and add pad templates for our sink and source pad. These
    // are later used for actually creating the pads and beforehand
    // already provide information to GStreamer about all possible
    // pads that could exist for this type.
    //
    // Our element here can convert BGRx to BGRx or GRAY8, both being grayscale.
    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            // On the src pad, we can produce BGRx and GRAY8 of any
            // width/height and with any framerate
            let caps = gst::Caps::builder("video/x-raw")
                .field(
                    "format",
                    gst::List::new([
                        gst_video::VideoFormat::Bgrx.to_str(),
                        gst_video::VideoFormat::Gray8.to_str(),
                    ]),
                )
                .field("width", gst::IntRange::new(0, i32::MAX))
                .field("height", gst::IntRange::new(0, i32::MAX))
                .field(
                    "framerate",
                    gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
                .build();
            // The src pad template must be named "src" for basetransform
            // and specific a pad that is always there
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let caps = gst::Caps::builder("video/x-raw")
                .field("format", gst_video::VideoFormat::Bgrx.to_str())
                .field("width", gst::IntRange::new(0, i32::MAX))
                .field("height", gst::IntRange::new(0, i32::MAX))
                .field(
                    "framerate",
                    gst::FractionRange::new(
                        gst::Fraction::new(0, 1),
                        gst::Fraction::new(i32::MAX, 1),
                    ),
                )
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

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

// Implementation of gst_base::BaseTransform virtual methods
impl BaseTransformImpl for AaInfer {
    // Configure basetransform so that we are always running in-place,
    // don't passthrough on same caps and also always call transform_ip
    // in passthrough mode (which does not matter for us here).
    //
    // We could work in-place for BGRx->BGRx but don't do here for simplicity
    // for now.
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    // Called for converting caps from one pad to another to account for any
    // changes in the media format this element is performing.
    //
    // In our case that means that:
    fn transform_caps(
        &self,
        element: &Self::Type,
        direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_caps = caps.clone();

        debug!(
            CAT,
            obj: element,
            "Transformed caps from {} to {} in direction {:?}",
            caps,
            other_caps,
            direction
        );

        // In the end we need to filter the caps through an optional filter caps to get rid of any
        // unwanted caps.
        if let Some(filter) = filter {
            Some(filter.intersect_with_mode(&other_caps, gst::CapsIntersectMode::First))
        } else {
            Some(other_caps)
        }
    }
}

impl VideoFilterImpl for AaInfer {
    fn transform_frame_ip(
        &self,
        element: &Self::Type,
        frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        // Keep the various metadata we need for working with the video frames in
        // local variables. This saves some typing below.
        let width = frame.width() as usize;
        let in_stride = frame.plane_stride()[0] as usize;
        // let in_data = frame.plane_data(0).unwrap();

        debug!(
            CAT,
            obj: element,
            "Transforming frame in-place, width={} in_stride={}",
            width,
            in_stride,
        );

        // // First check the output format. Our input format is always BGRx but the output might
        // // be BGRx or GRAY8. Based on what it is we need to do processing slightly differently.
        // if out_format == gst_video::VideoFormat::Bgrx {
        //     // Some assertions about our assumptions how the data looks like. This is only there
        //     // to give some further information to the compiler, in case these can be used for
        //     // better optimizations of the resulting code.
        //     //
        //     // If any of the assertions were not true, the code below would fail cleanly.
        //     assert_eq!(in_data.len() % 4, 0);
        //     assert_eq!(out_data.len() % 4, 0);
        //     assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

        //     let in_line_bytes = width * 4;
        //     let out_line_bytes = width * 4;

        //     assert!(in_line_bytes <= in_stride);
        //     assert!(out_line_bytes <= out_stride);

        //     // Iterate over each line of the input and output frame, mutable for the output frame.
        //     // Each input line has in_stride bytes, each output line out_stride. We use the
        //     // chunks_exact/chunks_exact_mut iterators here for getting a chunks of that many bytes per
        //     // iteration and zip them together to have access to both at the same time.
        //     for (in_line, out_line) in in_data
        //         .chunks_exact(in_stride)
        //         .zip(out_data.chunks_exact_mut(out_stride))
        //     {
        //         // Next iterate the same way over each actual pixel in each line. Every pixel is 4
        //         // bytes in the input and output, so we again use the chunks_exact/chunks_exact_mut iterators
        //         // to give us each pixel individually and zip them together.
        //         //
        //         // Note that we take a sub-slice of the whole lines: each line can contain an
        //         // arbitrary amount of padding at the end (e.g. for alignment purposes) and we
        //         // don't want to process that padding.
        //         for (in_p, out_p) in in_line[..in_line_bytes]
        //             .chunks_exact(4)
        //             .zip(out_line[..out_line_bytes].chunks_exact_mut(4))
        //         {
        //             assert_eq!(out_p.len(), 4);

        //             // Use our above-defined function to convert a BGRx pixel with the settings to
        //             // a grayscale value. Then store the same value in the red/green/blue component
        //             // of the pixel.
        //             let gray = Rgb2Gray::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
        //             out_p[0] = gray;
        //             out_p[1] = gray;
        //             out_p[2] = gray;
        //         }
        //     }
        // } else if out_format == gst_video::VideoFormat::Gray8 {
        //     assert_eq!(in_data.len() % 4, 0);
        //     assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

        //     let in_line_bytes = width * 4;
        //     let out_line_bytes = width;

        //     assert!(in_line_bytes <= in_stride);
        //     assert!(out_line_bytes <= out_stride);

        //     // Iterate over each line of the input and output frame, mutable for the output frame.
        //     // Each input line has in_stride bytes, each output line out_stride. We use the
        //     // chunks_exact/chunks_exact_mut iterators here for getting a chunks of that many bytes per
        //     // iteration and zip them together to have access to both at the same time.
        //     for (in_line, out_line) in in_data
        //         .chunks_exact(in_stride)
        //         .zip(out_data.chunks_exact_mut(out_stride))
        //     {
        //         // Next iterate the same way over each actual pixel in each line. Every pixel is 4
        //         // bytes in the input and 1 byte in the output, so we again use the
        //         // chunks_exact/chunks_exact_mut iterators to give us each pixel individually and zip them
        //         // together.
        //         //
        //         // Note that we take a sub-slice of the whole lines: each line can contain an
        //         // arbitrary amount of padding at the end (e.g. for alignment purposes) and we
        //         // don't want to process that padding.
        //         for (in_p, out_p) in in_line[..in_line_bytes]
        //             .chunks_exact(4)
        //             .zip(out_line[..out_line_bytes].iter_mut())
        //         {
        //             // Use our above-defined function to convert a BGRx pixel with the settings to
        //             // a grayscale value. Then store the value in the grayscale output directly.
        //             let gray = Rgb2Gray::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
        //             *out_p = gray;
        //         }
        //     }
        // } else {
        //     unimplemented!();
        // }

        Ok(gst::FlowSuccess::Ok)
    }
}
