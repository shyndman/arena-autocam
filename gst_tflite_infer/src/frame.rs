use std::fmt::Display;

use gst::Buffer;
use gst_video::{VideoFormat, VideoInfo};
use tflite_support::{TfLiteFrameBuffer, TfLiteFrameBufferFormat};

#[derive(Clone)]
/// This wrapper type is required so we can define a TryInto implementation
pub struct FrameBufferWrapper {
    pub video_info: VideoInfo,
    pub buffer: Buffer,
}

impl TryFrom<FrameBufferWrapper> for TfLiteFrameBuffer {
    type Error = FrameConvertError;
    fn try_from(value: FrameBufferWrapper) -> Result<Self, Self::Error> {
        let FrameBufferWrapper { video_info, buffer } = value;
        let mapped_buffer = buffer.map_readable().unwrap();
        let slice = &mut vec![0; buffer.size()];
        slice.copy_from_slice(mapped_buffer.as_slice());

        let src_format = video_info.format();

        let dst_format = match src_format {
            VideoFormat::I420 => TfLiteFrameBufferFormat::kYV21,
            VideoFormat::A420 => TfLiteFrameBufferFormat::kYV12,
            VideoFormat::Nv12 => TfLiteFrameBufferFormat::kNV12,
            VideoFormat::Nv21 => TfLiteFrameBufferFormat::kNV21,
            VideoFormat::Rgb => TfLiteFrameBufferFormat::kRGB,
            VideoFormat::Rgba => TfLiteFrameBufferFormat::kRGBA,
            VideoFormat::Gray8 => TfLiteFrameBufferFormat::kGRAY,
            _ => return Err(FrameConvertError { src_format }),
        };

        Ok(TfLiteFrameBuffer {
            format: dst_format,
            orientation: tflite_support::TfLiteFrameBufferOrientation::kTopLeft,
            dimension: tflite_support::TfLiteFrameBufferDimension {
                width: video_info.width() as i32,
                height: video_info.height() as i32,
            },
            buffer: slice.as_mut_ptr(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FrameConvertError {
    pub src_format: VideoFormat,
}

impl Display for FrameConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not handle source frame format, {}",
            self.src_format
        )
    }
}
