# Copyright 2021 The TensorFlow Authors. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""Main script to run the object detection routine."""
import argparse
import os
import subprocess
import time

import cv2
import libcamera
from picamera2 import MappedArray, Picamera2, Preview
from picamera2.encoders import H264Encoder
from picamera2.outputs import FfmpegOutput
from tflite_support.task import core, processor, vision
import tflite_runtime.interpreter as tflite

import utils

# Resolve a conflict between OpenCV and QT
os.environ.pop("QT_QPA_PLATFORM_PLUGIN_PATH")


def run(
    model: str,
    main_size: tuple[int, int] = (1640, 1232),
    lores_size: tuple[int, int] = (320, 240),
    num_threads: int = 4,
    show_preview=False,
    stream_config={},
) -> None:
    """Continuously run inference on images acquired from the camera.

    Args:
      model: Name of the TFLite object detection model.
      main_size: The size of the camera's main stream.
      lores_size: The size of the low resolution stream used for inference.
      num_threads: The number of CPU threads to run the model.
    """

    # Variables to calculate FPS
    counter, fps = 0, 0
    start_time = time.time()

    # Start capturing video input from the camera
    picam2 = Picamera2()
    if show_preview:
        picam2.start_preview(Preview.QT)
    config = picam2.create_video_configuration(
        main={"size": main_size},
        lores={"size": lores_size, "format": "YUV420"},
        transform=libcamera.Transform(hflip=True, vflip=True),
    )
    picam2.configure(config)
    print(config)

    # Configure the network stream
    encoder = H264Encoder(bitrate=1000000, repeat=True, iperiod=250)
    encoder.output = [FfmpegOutput("-f mpegts tcp://Ubuntu-Desktop.local:8554")]
    picam2.encoder = encoder
    picam2.start_encoder()
    picam2.start()

    # Visualization parameters
    fps_avg_frame_count = 10

    # Initialize the object detection model
    base_options = core.BaseOptions(
        file_name=model,
        use_coral=False,
        num_threads=num_threads,
    )
    detection_options = processor.DetectionOptions(max_results=3, score_threshold=0.28)
    options = vision.ObjectDetectorOptions(
        base_options=base_options, detection_options=detection_options
    )
    detector = vision.ObjectDetector.create_from_options(options)

    # Continuously capture images from the camera and run inference
    while True:
        counter += 1

        image = picam2.capture_array("lores")

        # Convert the image from YUV to RGB as required by the TFLite model.
        rgb_image = cv2.cvtColor(image, cv2.COLOR_YUV420p2RGB)

        # Create a TensorImage object from the RGB image.
        input_tensor = vision.TensorImage.create_from_array(rgb_image)

        # Run object detection estimation using the model.
        detection_result = detector.detect(input_tensor)

        # # Draw keypoints and edges on input image
        # image = utils.visualize(image, detection_result)

        # Calculate the FPS
        if counter % fps_avg_frame_count == 0:
            end_time = time.time()
            fps = fps_avg_frame_count / (end_time - start_time)
            start_time = time.time()

            print(f"fps: {fps}")

        # # Show the FPS
        # fps_text = "FPS = {:.1f}".format(fps)
        # text_location = (left_margin, row_size)
        # cv2.putText(
        #     image,
        #     fps_text,
        #     text_location,
        #     cv2.FONT_HERSHEY_PLAIN,
        #     font_size,
        #     text_color,
        #     font_thickness,
        # )

        # # Stop the program if the ESC key is pressed.
        # if cv2.waitKey(1) == 27:
        #     break
        # cv2.imshow("object_detector", image)

    # cap.release()
    # cv2.destroyAllWindows()


def begin_video_stream():
    # Start the gstreamer process with a pipe for stdin
    return subprocess.Popen(
        [
            "cvlc",
            "-vvv",
            "stream:///dev/stdin",
            "--sout",
            "#rtp{dst=192.168.86.165,port=8554,sdp=http://192.168.86.165:8080/stream}",
            ":demux=h264",
        ],
        stdin=subprocess.PIPE,
    )


def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )
    parser.add_argument(
        "--model",
        help="Path of the object detection model.",
        required=False,
        default="horsies.tflite",
    )
    parser.add_argument(
        "--num_threads",
        help="Number of CPU threads to run the model.",
        required=False,
        type=int,
        default=4,
    )
    parser.add_argument(
        "--preview",
        help="True to show a preview window",
        required=False,
        type=bool,
        default=False,
        action=argparse.BooleanOptionalAction,
    )
    args = parser.parse_args()

    run(args.model, num_threads=args.num_threads, show_preview=args.preview)


if __name__ == "__main__":
    main()
