use crate::{
    bindings::{
        TfLiteCategory, TfLiteDetection, TfLiteDetectionResult, TfLiteDetectionResultDelete,
        TfLiteFrameBuffer, TfLiteObjectDetector, TfLiteObjectDetectorDelete,
        TfLiteObjectDetectorDetect, TfLiteObjectDetectorFromOptions,
        TfLiteObjectDetectorOptions, TfLiteObjectDetectorOptionsCreate, TfLiteSupportError,
        TfLiteSupportErrorCode, TfLiteSupportErrorDelete,
    },
    TfLiteFrameBufferDimension, TfLiteFrameBufferFormat, TfLiteFrameBufferOrientation,
};
use std::{
    ffi::{CStr, CString},
    ptr::null_mut,
    slice::from_raw_parts,
};

pub struct BaseOptions {
    pub model_path: String,
    pub num_threads: i32,
}

pub struct DetectionOptions {
    // score_threshold: Optional[float] = None,
    pub score_threshold: Option<f32>,
    // max_results: Optional[int] = None
    pub max_results: Option<i32>,
    // category_name_allowlist: Optional[List[str]] = None,
    // category_name_denylist: Optional[List[str]] = None,
    // display_names_locale: Optional[str] = None,
}

#[derive(Debug)]

pub struct ObjectDetector {
    native_detector: *mut TfLiteObjectDetector,
}

impl ObjectDetector {
    pub fn with_options(
        base_options: BaseOptions,
        detection_options: DetectionOptions,
    ) -> Result<ObjectDetector> {
        unsafe {
            let mut native_options: TfLiteObjectDetectorOptions =
                TfLiteObjectDetectorOptionsCreate();
            native_options
                .base_options
                .compute_settings
                .cpu_settings
                .num_threads = base_options.num_threads;

            let file_path = CString::new(base_options.model_path).unwrap();
            native_options.base_options.model_file.file_path = file_path.as_ptr();
            native_options.classification_options.max_results = detection_options
                .max_results
                .unwrap_or(native_options.classification_options.max_results);
            native_options.classification_options.score_threshold = detection_options
                .score_threshold
                .unwrap_or(native_options.classification_options.score_threshold);

            let mut err: *mut TfLiteSupportError = null_mut();
            let native_detector = TfLiteObjectDetectorFromOptions(&native_options, &mut err);
            if !err.is_null() {
                let rust_error = Error {
                    code: (*err).code,
                    message: {
                        let err_cstr = CStr::from_ptr((*err).message);
                        err_cstr.to_str().unwrap().into()
                    },
                };
                TfLiteSupportErrorDelete(err);

                return Err(rust_error);
            }

            return Ok(ObjectDetector { native_detector });
        }
    }

    pub fn detect<'a, IntoFrame>(&self, frame: &IntoFrame) -> Result<DetectionResult>
    where
        IntoFrame: Copy + Into<&'a TfLiteFrameBuffer>,
    {
        unsafe {
            let mut err: *mut TfLiteSupportError = null_mut();
            let frame_buffer: &TfLiteFrameBuffer = (*frame).into();
            let native_result = TfLiteObjectDetectorDetect(
                self.native_detector,
                frame_buffer as *const TfLiteFrameBuffer,
                &mut err,
            );

            if !err.is_null() {
                eprintln!("Error running detect: {:?}", (*err).code);

                TfLiteSupportErrorDelete(err);
            }

            return Ok(DetectionResult { native_result });
        }
    }
}

impl Drop for ObjectDetector {
    fn drop(&mut self) {
        unsafe {
            TfLiteObjectDetectorDelete(self.native_detector);
        }
    }
}

pub struct DetectionResult {
    native_result: *mut TfLiteDetectionResult,
}

impl DetectionResult {
    pub fn detections(&self) -> impl Iterator<Item = Detection> {
        let native_detections = unsafe {
            from_raw_parts(
                (*self.native_result).detections as *const TfLiteDetection,
                (*self.native_result).size as usize,
            )
        };

        native_detections
            .into_iter()
            .map(|native_detection| Detection { native_detection })
    }
}

impl Drop for DetectionResult {
    fn drop(&mut self) {
        unsafe {
            TfLiteDetectionResultDelete(self.native_result);
        }
    }
}

pub struct Detection {
    native_detection: *const TfLiteDetection,
}

impl Detection {
    pub fn bounding_box(&self) -> Rect {
        unsafe {
            Rect {
                x: (*self.native_detection).bounding_box.origin_x,
                y: (*self.native_detection).bounding_box.origin_y,
                width: (*self.native_detection).bounding_box.width,
                height: (*self.native_detection).bounding_box.height,
            }
        }
    }

    pub fn categories(&self) -> impl Iterator<Item = DetectionCategory> {
        let native_categories = unsafe {
            from_raw_parts(
                (*self.native_detection).categories as *const TfLiteCategory,
                (*self.native_detection).size as usize,
            )
        };

        native_categories
            .into_iter()
            .map(|c| DetectionCategory { native_category: c })
    }
}

// The category of an object detection.
pub struct DetectionCategory {
    // No need to manually delete this, as it is destroyed along with the
    // TfLiteDetectionResult
    native_category: *const TfLiteCategory,
}

impl DetectionCategory {
    pub fn index(&self) -> usize {
        (unsafe { (*self.native_category).index }) as usize
    }

    pub fn score(&self) -> f32 {
        unsafe { (*self.native_category).score }
    }

    pub fn label(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr((*self.native_category).label) };
        c_str.to_str().unwrap()
    }

    pub fn display_name(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr((*self.native_category).display_name) };
        c_str.to_str().unwrap()
    }
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub type FrameBufferFormat = TfLiteFrameBufferFormat;
pub type FrameBufferOrientation = TfLiteFrameBufferOrientation;

pub struct FrameBuffer {
    pub format: FrameBufferFormat,
    pub orientation: FrameBufferOrientation,
    pub dimension: (i32, i32),
    pub buffer: Box<[u8]>,
}

impl FrameBuffer {
    fn to_tflite_frame(&self) -> TfLiteFrameBuffer {
        TfLiteFrameBuffer {
            format: self.format,
            orientation: self.orientation,
            dimension: TfLiteFrameBufferDimension {
                width: self.dimension.0,
                height: self.dimension.1,
            },
            buffer: self.buffer.clone().as_mut_ptr(),
        }
    }
}

impl From<FrameBuffer> for TfLiteFrameBuffer {
    fn from(value: FrameBuffer) -> Self {
        value.to_tflite_frame()
    }
}

/// A specialized [`Result`] type for API operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for TensorFlow Lite operations.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Error {
    // TODO(shyndman): Have this mirror the support error type
    pub code: TfLiteSupportErrorCode,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{BaseOptions, DetectionOptions, ObjectDetector};

    const MODEL_PATH: &'static str = "tests/sample.tflite";

    #[test]
    fn test_detector_with_model_path() {
        let base_opts = BaseOptions {
            model_path: MODEL_PATH.into(),
            num_threads: 3,
        };
        let detection_opts = DetectionOptions {
            max_results: Some(5),
            score_threshold: Some(0.7),
        };

        match ObjectDetector::with_options(base_opts, detection_opts) {
            Ok(detector) => println!("Got a detector {:?}", detector),
            Err(error) => println!("Error: {:?}", error),
        }
    }
}
