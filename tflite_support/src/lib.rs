mod object_detector;

pub(crate) mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    TfLiteFrameBuffer, TfLiteFrameBufferDimension, TfLiteFrameBufferFormat,
    TfLiteFrameBufferOrientation,
};
pub use object_detector::*;
