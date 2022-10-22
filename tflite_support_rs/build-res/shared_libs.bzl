"""
Rules for building tensorflow_lite_support into C shared libraries for binding
to Rust.
"""

load(
    "@org_tensorflow//tensorflow/lite:build_def.bzl",
    "tflite_cc_shared_object",
)

tflite_cc_shared_object(
    name = "object_detector_c",
    per_os_targets = True,
    deps = select({
        # Coral Edge TPU support
        ":darwinn_portable": ["//tensorflow_lite_support/acceleration/configuration:edgetpu_coral_plugin"],
        "//conditions:default": [],
    }) + select({
        # XNNPack Support
        ":tflite_with_xnnpack": ["//tensorflow_lite_support/acceleration/configuration:xnnpack_plugin"],
        "//conditions:default": [],
    }) + [
        # ":c_api_with_xnn_pack",
        "//tensorflow_lite_support/c/task/vision:object_detector",
        "@org_tensorflow//tensorflow/lite/c:exported_symbols.lds",
        "@org_tensorflow//tensorflow/lite/c:version_script.lds",
    ],
)

config_setting(
    name = "darwinn_portable",
    values = {
        "define": "darwinn_portable=1",
    },
)

config_setting(
    name = "tflite_with_xnnpack",
    values = {
        "define": "tflite_with_xnnpack=true",
    },
)

# cc_library(
#     name = "c_api_with_xnn_pack",
#     hdrs = ["//tensorflow/lite/c:c_api.h",
#             "//tensorflow/lite/delegates/xnnpack:xnnpack_delegate.h"],
#     copts = tflite_copts(),
#     deps = [
#         "//tensorflow/lite/c:c_api",
#         "//tensorflow/lite/delegates/xnnpack:xnnpack_delegate"
#     ],
#     alwayslink = 1,
# )
