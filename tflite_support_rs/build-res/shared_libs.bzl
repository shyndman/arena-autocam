"""
Rules for building tensorflow_lite_support into C shared libraries for binding
to Rust.
"""

load(
    "@org_tensorflow//tensorflow/lite:build_def.bzl",
    "tflite_cc_shared_object",
)

tflite_cc_shared_object(
    name = "tensorflowlite_c",
    linkopts = select({
        "@org_tensorflow//tensorflow:ios": [
            "-Wl,-exported_symbols_list,$(location @org_tensorflow//tensorflow/lite/c:exported_symbols.lds)",
        ],
        "@org_tensorflow//tensorflow:macos": [
            "-Wl,-exported_symbols_list,$(location @org_tensorflow//tensorflow/lite/c:exported_symbols.lds)",
        ],
        "@org_tensorflow//tensorflow:windows": [],
        "//conditions:default": [
            "-z defs",
            "-Wl,--version-script,$(location @org_tensorflow//tensorflow/lite/c:version_script.lds)",
        ],
    }),
    per_os_targets = True,
    deps = [
        # ":c_api_with_xnn_pack",
        "@org_tensorflow//tensorflow/lite/c:c_api",
        "@org_tensorflow//tensorflow/lite/c:c_api_experimental",
        "@org_tensorflow//tensorflow/lite/c:exported_symbols.lds",
        "@org_tensorflow//tensorflow/lite/c:version_script.lds",
    ],
)

tflite_cc_shared_object(
    name = "object_detector_c",
    linkopts = select({
        "@org_tensorflow//tensorflow:ios": [
            "-Wl,-exported_symbols_list,$(location @org_tensorflow//tensorflow/lite/c:exported_symbols.lds)",
        ],
        "@org_tensorflow//tensorflow:macos": [
            "-Wl,-exported_symbols_list,$(location @org_tensorflow//tensorflow/lite/c:exported_symbols.lds)",
        ],
        "@org_tensorflow//tensorflow:windows": [],
        "//conditions:default": [
            "-z defs",
            "-Wl,--version-script,$(location @org_tensorflow//tensorflow/lite/c:version_script.lds)",
        ],
    }),
    per_os_targets = True,
    deps = [
        # ":c_api_with_xnn_pack",
        "//tensorflow_lite_support/c/task/vision:object_detector",
        "@org_tensorflow//tensorflow/lite/c:exported_symbols.lds",
        "@org_tensorflow//tensorflow/lite/c:version_script.lds",
    ],
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
