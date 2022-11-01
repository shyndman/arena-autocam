use anyhow::{anyhow, Error};
use gst::Buffer;
use gst_video::{VideoFormat, VideoInfo};
use tflite_support::{TfLiteFrameBuffer, TfLiteFrameBufferFormat};

#[derive(Clone)]
/// This wrapper type is required so we can define a TryInto implementation
pub struct TensorflowBufferAdapter<'frame> {
    pub video_info: &'frame VideoInfo,
    pub buffer: &'frame Buffer,
}

impl<'frame> TryFrom<TensorflowBufferAdapter<'frame>> for TfLiteFrameBuffer {
    type Error = Error;
    fn try_from(value: TensorflowBufferAdapter) -> Result<Self, Self::Error> {
        let TensorflowBufferAdapter { video_info, buffer } = value;

        let src_format = video_info.format();
        let dst_format = match src_format {
            VideoFormat::I420 => TfLiteFrameBufferFormat::kYV21,
            VideoFormat::A420 => TfLiteFrameBufferFormat::kYV12,
            VideoFormat::Nv12 => TfLiteFrameBufferFormat::kNV12,
            VideoFormat::Nv21 => TfLiteFrameBufferFormat::kNV21,
            VideoFormat::Rgb => TfLiteFrameBufferFormat::kRGB,
            VideoFormat::Rgba => TfLiteFrameBufferFormat::kRGBA,
            VideoFormat::Gray8 => TfLiteFrameBufferFormat::kGRAY,
            _ => {
                return Err(anyhow!(
                    "Cannot map between {} and the available destination formats",
                    src_format
                ));
            }
        };

        let mapped_reader = buffer.map_readable().unwrap();
        let buffer_pointer = mapped_reader.as_ptr() as *mut u8;

        Ok(TfLiteFrameBuffer {
            format: dst_format,
            orientation: tflite_support::TfLiteFrameBufferOrientation::kTopLeft,
            dimension: tflite_support::TfLiteFrameBufferDimension {
                width: video_info.width() as i32,
                height: video_info.height() as i32,
            },
            buffer: buffer_pointer,
        })
    }
}
