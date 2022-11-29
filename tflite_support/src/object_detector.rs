use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use anyhow::Result;

use crate::bindings::{
    TfLiteCategory, TfLiteDetection, TfLiteDetectionResult, TfLiteDetectionResultDelete,
    TfLiteFrameBuffer, TfLiteObjectDetector, TfLiteObjectDetectorDelete,
    TfLiteObjectDetectorDetect, TfLiteObjectDetectorFromOptions, TfLiteObjectDetectorOptions,
    TfLiteObjectDetectorOptionsCreate, TfLiteSupportError, TfLiteSupportErrorCode,
    TfLiteSupportErrorDelete,
};
use crate::error::Error;

#[derive(Debug, Default)]
pub struct BaseOptions {
    pub model_path: String,
    pub num_threads: i32,
    #[cfg(feature = "coral_tpu")]
    pub enable_coral: bool,
}

#[derive(Debug, Default)]
pub struct DetectionOptions {
    pub score_threshold: Option<f32>,
    pub max_results: Option<i32>,
    // TODO(shyndman): category_name_allowlist: Optional[List[str]] = None,
    // TODO(shyndman): category_name_denylist: Optional[List[str]] = None,
    // TODO(shyndman): display_names_locale: Optional[str] = None,
}

#[derive(Debug)]
pub struct ObjectDetector {
    native_detector: *mut TfLiteObjectDetector,
}
unsafe impl Send for ObjectDetector {}
unsafe impl Sync for ObjectDetector {}

impl ObjectDetector {
    pub fn with_options(
        base_options: BaseOptions,
        detection_options: DetectionOptions,
    ) -> Result<ObjectDetector, Error> {
        unsafe {
            let mut native_options: TfLiteObjectDetectorOptions =
                TfLiteObjectDetectorOptionsCreate();
            let mut compute_settings = native_options.base_options.compute_settings;
            compute_settings.cpu_settings.num_threads = base_options.num_threads;
            #[cfg(feature = "coral_tpu")]
            {
                compute_settings.coral_delegate_settings.enable_delegate =
                    base_options.enable_coral;
            }

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

    pub fn detect<IntoFrame>(&self, frame: IntoFrame) -> Result<DetectionResult, Error>
    where
        IntoFrame: TryInto<TfLiteFrameBuffer>,
    {
        let frame_buffer: TfLiteFrameBuffer = frame.try_into().map_err(|_e| Error {
            // TODO return something meaningful
            code: TfLiteSupportErrorCode::kError,
            message: "".into(),
        })?;

        unsafe {
            let mut err: *mut TfLiteSupportError = null_mut();
            let native_result = TfLiteObjectDetectorDetect(
                self.native_detector,
                &frame_buffer as *const TfLiteFrameBuffer,
                &mut err,
            );

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
    pub fn size(&self) -> usize {
        unsafe { (*self.native_result).size as usize }
    }

    pub fn detections(&self) -> impl Iterator<Item = Detection> {
        let native_detections = unsafe {
            from_raw_parts(
                (*self.native_result).detections as *const TfLiteDetection,
                self.size(),
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

    pub fn category_count(&self) -> usize {
        unsafe { (*self.native_detection).size as usize }
    }

    pub fn categories(&self) -> impl Iterator<Item = DetectionCategory> {
        let native_categories = unsafe {
            from_raw_parts(
                (*self.native_detection).categories as *const TfLiteCategory,
                self.category_count(),
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

impl Display for DetectionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DetectionCategory(score={} label={})",
            self.score(),
            self.label()
        )
    }
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

    pub fn label_as_string(&self) -> String {
        self.label().clone().into()
    }

    pub fn display_name(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr((*self.native_category).display_name) };
        c_str.to_str().unwrap()
    }
}

#[derive(Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(test)]
mod tests {
    extern crate findshlibs;
    use findshlibs::{Segment, SharedLibrary, TargetSharedLibrary};

    use super::{BaseOptions, DetectionOptions, ObjectDetector};

    const MODEL_PATH: &'static str = "tests/sample.tflite";

    #[test]
    fn test_detector_with_model_path() {
        TargetSharedLibrary::each(|shlib| {
            println!("{}", shlib.name().to_string_lossy());

            for seg in shlib.segments() {
                println!(
                    "    {}: segment {}",
                    seg.actual_virtual_memory_address(shlib),
                    seg.name()
                );
            }
        });

        let base_opts = BaseOptions {
            model_path: MODEL_PATH.into(),
            num_threads: 3,
            ..BaseOptions::default()
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
