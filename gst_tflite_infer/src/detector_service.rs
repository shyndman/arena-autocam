use std::time::Instant;

use gst::Buffer;
use gst_video::VideoInfo;
use once_cell::sync::Lazy;
use tflite_support::{BaseOptions, DetectionOptions, ObjectDetector};

use crate::{
    frame::FrameBufferWrapper,
    util::crossbeam_request::{channel, RequestReceiver, RequestSender},
};

pub struct InferenceRequest {
    vid_info: VideoInfo,
    buffer: Buffer,
}
pub struct InferenceResponse;

const MODEL_PATH: &'static str = "sample_data/detection-model.tflite";

static INFERENCE_CHANNEL: Lazy<RequestSender<InferenceRequest, InferenceResponse>> =
    Lazy::new(|| {
        let (send_req, receive_req) = channel();
        std::thread::spawn(move || run_inference_loop(receive_req));
        send_req
    });

pub fn detect_objects_in_frame(vid_info: VideoInfo, buffer: Buffer) -> InferenceResponse {
    let response_receiver = match INFERENCE_CHANNEL.request(InferenceRequest {
        vid_info: vid_info,
        buffer: buffer,
    }) {
        Ok(response_receiver) => response_receiver,
        Err(_err) => panic!(),
    };

    match response_receiver.collect() {
        Ok(response) => return response,
        Err(_err) => todo!(),
    }
}

fn run_inference_loop(receive_req: RequestReceiver<InferenceRequest, InferenceResponse>) {
    let base_opts = BaseOptions {
        model_path: MODEL_PATH.into(),
        num_threads: 4,
        ..BaseOptions::default()
    };
    let detection_opts = DetectionOptions {
        max_results: Some(5),
        score_threshold: Some(0.0),
    };
    let detector = match ObjectDetector::with_options(base_opts, detection_opts) {
        Ok(detector) => detector,
        Err(error) => panic!("Error: {:?}", error),
    };

    loop {
        receive_req.poll_loop(|request, response_sender| {
            let ts = Instant::now();
            let wrapper = FrameBufferWrapper {
                video_info: request.vid_info,
                buffer: request.buffer,
            };
            let detection_result = detector
                .detect(wrapper)
                .expect("Failed to run the object detector");

            eprintln!(
                "Detection finished, {} objects detected in {}ms",
                detection_result.size(),
                ts.elapsed().as_millis()
            );

            response_sender.respond(InferenceResponse);

            // for (i, detection) in detection_result.detections().enumerate() {
            //     eprintln!("detection #{}", i);
            //     eprintln!("bounds={:#?}", detection.bounding_box());

            //     let category_strings: Vec<String> =
            //         detection.categories().map(|c| format!("{}", c)).collect();
            //     eprintln!("categories={:?}", category_strings.join(", "));
            // }
        });
        // sleep(Duration::from_millis(100));
    }
}
