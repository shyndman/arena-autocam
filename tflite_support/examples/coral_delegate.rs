extern crate findshlibs;

use findshlibs::{Segment, SharedLibrary, TargetSharedLibrary};
use tflite_support::{BaseOptions, DetectionOptions, ObjectDetector};
const MODEL_PATH: &'static str = "sample_data/detection-model.tflite";

fn main() {
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
        enable_coral: true,
        ..BaseOptions::default()
    };
    let detection_opts = DetectionOptions {
        max_results: Some(5),
        score_threshold: Some(0.7),
    };

    let _detector = match ObjectDetector::with_options(base_opts, detection_opts) {
        Ok(detector) => {println!("Got a detector {:?}", detector); detector},
        Err(error) => panic!("Error: {:?}", error),
    };

    // detector.detect(frame)
}
