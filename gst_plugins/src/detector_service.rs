use crate::{
    frame::FrameBufferWrapper,
    util::crossbeam_request::{channel, RequestReceiver, RequestSender},
};
use gst::Buffer;
use gst_video::VideoInfo;
use once_cell::sync::Lazy;
use std::thread::sleep;
use std::time::Duration;
use tflite_support::{BaseOptions, DetectionOptions, ObjectDetector};

pub struct InferenceRequest {
    vid_info: VideoInfo,
    buffer: Buffer,
}
pub struct InferenceResponse;

const MODEL_PATH: &'static str = "test/sample.tflite";

static INFERENCE_CHANNEL: Lazy<RequestSender<InferenceRequest, InferenceResponse>> =
    Lazy::new(|| {
        let vlog_level = std::env::var("TF_CPP_MAX_VLOG_LEVEL").expect("NOT HERE!!!!");
        eprintln!("@@@{}@@@", vlog_level);

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
        num_threads: 3,
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
            let wrapper = FrameBufferWrapper {
                video_info: request.vid_info,
                buffer: request.buffer,
            };
            let detection_result = detector.detect(wrapper).expect("That this works");

            eprintln!(
                "Detection finished, {} objects detected",
                detection_result.size()
            );

            for (i, detection) in detection_result.detections().enumerate() {
                eprintln!("detection #{}", i);
                eprintln!("bounds={:#?}", detection.bounding_box());

                let category_strings: Vec<String> =
                    detection.categories().map(|c| format!("{}", c)).collect();
                eprintln!("categories={:?}", category_strings.join(", "));
            }

            response_sender.respond(InferenceResponse);
        });
        sleep(Duration::from_millis(100));
    }
}
