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
    linkopts = [
        "-Wl,-z,defs",
        "-Wl,--version-script,$(location :tflite_support_version_script.lds)",
    ],
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
        ":tflite_support_version_script.lds",
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
