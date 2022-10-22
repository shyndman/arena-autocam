#![feature(const_mut_refs)]

extern crate bindgen;

use std::env;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Instant;

use const_format::formatc;

const TFLITE_SUPPORT_GIT_URL: &str = "https://github.com/shyndman/tflite-support.git";
const TFLITE_SUPPORT_GIT_TAG: &str = "v0.4.3+scott";
const BAZEL_COPTS_ENV_VAR: &str = "TFLITEC_BAZEL_COPTS";
const PREBUILT_PATH_ENV_VAR: &str = "TFLITEC_PREBUILT_PATH";

const SHARED_LIB_STEM: &str = "object_detector_c";
const SHARED_LIB_REL_PATH: &str = "tensorflow_lite_support/c/bindgen";
const SHARED_LIB_BAZEL_TARGET: &str = formatc!("//{SHARED_LIB_REL_PATH}:object_detector_c");

const CONFIGURE_TARGET: &str = "@org_tensorflow//:configure.py";

fn target_os() -> String {
    env::var("CARGO_CFG_TARGET_OS").expect("Unable to get CARGO_CFG_TARGET_OS")
}

fn target_arch() -> String {
    env::var("CARGO_CFG_TARGET_ARCH").expect("Unable to get CARGO_CFG_TARGET_ARCH")
}

fn is_debug_build() -> bool {
    env::var("DEBUG").unwrap_or("0".into()).as_str() != "0"
}

fn dll_extension() -> &'static str {
    match target_os().as_str() {
        "macos" => "dylib",
        "windows" => "dll",
        _ => "so",
    }
}

fn dll_prefix() -> &'static str {
    match target_os().as_str() {
        "windows" => "",
        _ => "lib",
    }
}

fn generate_bindgen_layout_tests() -> bool {
    env::var("BINDGEN_TESTS").unwrap_or("0".into()) == "1"
}

fn normalized_target() -> Option<String> {
    env::var("TARGET")
        .ok()
        .map(|t| t.to_uppercase().replace('-', "_"))
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn main() {
    eprintln!("Building TfLite Support");
    eprintln!("is_debug: {}", is_debug_build());
    eprintln!("target_arch: {}", target_arch());

    println!("cargo:rerun-if-env-changed={}", BAZEL_COPTS_ENV_VAR);
    println!("cargo:rerun-if-env-changed={}", PREBUILT_PATH_ENV_VAR);
    if let Some(target) = normalized_target() {
        println!(
            "cargo:rerun-if-env-changed={}_{}",
            PREBUILT_PATH_ENV_VAR, target
        );
    }

    let out_path = out_dir();
    println!("cargo:rustc-link-search=native={}", out_path.display());
    println!("cargo:rustc-link-lib=dylib=object_detector_c");

    // Build from source
    let config = target_os();
    let tf_src_path = out_path.join(format!("tflite_support_{}", TFLITE_SUPPORT_GIT_TAG));

    check_and_set_envs();
    prepare_tensorflow_source(tf_src_path.as_path());
    configure_build(tf_src_path.as_path());
    build_tensorflow_with_bazel(tf_src_path.to_str().unwrap(), config.as_str());

    // Generate bindings using headers
    generate_bindings(tf_src_path);
}

fn check_and_set_envs() {
    let python_bin_path =
        get_python_bin_path().expect("Cannot find Python binary having required packages.");
    let default_envs = [
        ["PYTHON_BIN_PATH", python_bin_path.to_str().unwrap()],
        ["USE_DEFAULT_PYTHON_LIB_PATH", "1"],
        ["TF_NEED_OPENCL", "0"],
        ["TF_CUDA_CLANG", "0"],
        ["TF_NEED_TENSORRT", "0"],
        ["TF_DOWNLOAD_CLANG", "0"],
        ["TF_NEED_MPI", "0"],
        ["TF_NEED_ROCM", "0"],
        ["TF_NEED_CUDA", "0"],
        ["TF_OVERRIDE_EIGEN_STRONG_INLINE", "1"], // Windows only
        ["CC_OPT_FLAGS", "-Wno-sign-compare"],
        ["TF_SET_ANDROID_WORKSPACE", "0"],
        ["TF_CONFIGURE_IOS", "0"],
    ];
    for kv in default_envs {
        let name = kv[0];
        let val = kv[1];
        if env::var(name).is_err() {
            env::set_var(name, val);
        }
    }
    let true_vals = ["1", "t", "true", "y", "yes"];
    if true_vals.contains(&env::var("TF_SET_ANDROID_WORKSPACE").unwrap().as_str()) {
        let android_env_vars = [
            "ANDROID_NDK_HOME",
            "ANDROID_NDK_API_LEVEL",
            "ANDROID_SDK_HOME",
            "ANDROID_API_LEVEL",
            "ANDROID_BUILD_TOOLS_VERSION",
        ];
        for name in android_env_vars {
            env::var(name)
                .unwrap_or_else(|_| panic!("{} should be set to build for Android", name));
        }
    }
}

fn prepare_tensorflow_source(tf_src_path: &Path) {
    let complete_clone_hint_file = tf_src_path.join(".complete_clone");
    if complete_clone_hint_file.exists() {
        return;
    }

    if tf_src_path.exists() {
        std::fs::remove_dir_all(tf_src_path).expect("Cannot clean tf_src_path");
    }
    let mut git = std::process::Command::new("git");
    git.arg("clone")
        .args(&["--depth", "1"])
        .arg("--shallow-submodules")
        .args(&["--branch", TFLITE_SUPPORT_GIT_TAG])
        .arg("--single-branch")
        .arg(TFLITE_SUPPORT_GIT_URL)
        .arg(tf_src_path.to_str().unwrap());
    println!("Git clone started");
    let start = Instant::now();
    if !git.status().expect("Cannot execute `git clone`").success() {
        panic!("git clone failed");
    }
    std::fs::File::create(complete_clone_hint_file).expect("Cannot create clone hint file!");
    println!("Clone took {:?}", Instant::now() - start);

    let bindgen_path = tf_src_path.join(SHARED_LIB_REL_PATH);
    eprintln!(
        "Creating new build directory in source tree, {:?}",
        bindgen_path
    );
    std::fs::create_dir_all(bindgen_path.clone()).expect("Unable to create build directory");

    let bindgen_build_path = bindgen_path.join("BUILD");
    eprintln!(
        "Copying custom BUILD file into tflite_support source tree, dest={:?}",
        bindgen_build_path
    );
    copy_or_overwrite(
        PathBuf::from("build-res/shared_libs.bzl"),
        bindgen_build_path,
    );
}

fn configure_build(tf_src_path: &Path) {
    // Grab the configure.py file from the tensorflow repo
    std::process::Command::new("bazel")
        .arg("build")
        .arg(CONFIGURE_TARGET)
        .current_dir(tf_src_path)
        .status()
        .expect("Could not execute bazel");

    // Copy it over to the repo root
    let query_out = unsafe {
        String::from_utf8_unchecked(
            std::process::Command::new("bazel")
                .arg("query")
                .arg("--output")
                .arg("location")
                .arg(CONFIGURE_TARGET)
                .current_dir(tf_src_path)
                .output()
                .expect("Could not query path of configure.py")
                .stdout,
        )
    };
    let configure_path = query_out
        .get(
            ..query_out
                .find(":")
                .expect("bazel query did not return expected location formation"),
        )
        .unwrap();
    copy_or_overwrite(configure_path, tf_src_path.join("configure.py"));

    // Invoke configure.py -- it will run automatically because of the env vars we've set
    std::fs::create_dir_all(tf_src_path.join("tools"))
        .expect("Could not create tools directory (required by configure.py)");
    let python_bin_path = env::var("PYTHON_BIN_PATH").expect("Cannot read PYTHON_BIN_PATH");
    if !std::process::Command::new(&python_bin_path)
        .arg("configure.py")
        .current_dir(tf_src_path)
        .status()
        .unwrap_or_else(|_| panic!("Cannot execute python at {}", &python_bin_path))
        .success()
    {
        panic!("Cannot configure tensorflow")
    }
}

fn build_tensorflow_with_bazel(tf_src_path: &str, bazel_config_option: &str) {
    let libc_out_dir = PathBuf::from(tf_src_path)
        .join("bazel-bin")
        .join("tensorflow_lite_support")
        .join("c")
        .join("bindgen");

    let shared_lib_basename =
        format!("{}{SHARED_LIB_STEM}.{}", dll_prefix(), dll_extension());
    let output_shared_lib_path = libc_out_dir.join(shared_lib_basename.clone());

    let mut bazel = std::process::Command::new("bazel");
    bazel
        .arg("--bazelrc")
        .arg(".tf_configure.bazelrc")
        .arg("build")
        // We always want verbose failures
        .arg("--verbose_failures");

    if is_debug_build() {
        bazel
            .arg("--config")
            .arg("dbg")
            .arg("--config")
            .arg("verbose_logs")
            .arg("--subcommands");
    }

    let arch = target_arch();
    match arch.as_str() {
        "aarch64" => {
            bazel.arg("--config").arg("elinux_aarch64");
        }
        _ => {}
    };

    // Configure XNNPACK flags
    #[cfg(not(feature = "xnnpack"))]
    bazel.arg("--define").arg("tflite_with_xnnpack=false");
    #[cfg(any(feature = "xnnpack_qu8", feature = "xnnpack_qs8"))]
    bazel.arg("--define").arg("tflite_with_xnnpack=true");
    #[cfg(feature = "xnnpack_qs8")]
    bazel.arg("--define").arg("xnn_enable_qs8=true");
    #[cfg(feature = "xnnpack_qu8")]
    bazel.arg("--define").arg("xnn_enable_qu8=true");

    // Configure Coral TPU flags
    #[cfg(feature = "coral_tpu")]
    bazel
        .arg("--define")
        .arg("darwinn_portable=1")
        .arg("--linkopt")
        .arg("-lusb-1.0")
        .arg("--linkopt")
        .arg("-L/usr/lib/aarch64-linux-gnu/");

    bazel
        .arg(format!("--config={}", bazel_config_option))
        .arg(SHARED_LIB_BAZEL_TARGET)
        .current_dir(tf_src_path);

    bazel.arg("--copt").arg("-frecord-gcc-switches");
    if let Ok(copts) = env::var(BAZEL_COPTS_ENV_VAR) {
        let copts = copts.split_ascii_whitespace();
        for opt in copts {
            bazel.arg(format!("--copt={}", opt));
        }
    }

    eprintln!("Bazel Build Command: {:?}", bazel);
    if !bazel.status().expect("Cannot execute bazel").success() {
        panic!("Cannot build Tensorflow Lite Support");
    }

    let c_lib_out = out_dir().join("libobject_detector_c.so");
    copy_or_overwrite(&output_shared_lib_path, &c_lib_out);
}

fn generate_bindings(tf_src_path: PathBuf) {
    let builder = bindgen::Builder::default().header(
        tf_src_path
            .join("tensorflow_lite_support/c/task/vision/object_detector.h")
            .to_str()
            .unwrap(),
    );

    let bindings = builder
        // Set the root of the source tree as an include path
        .clang_arg(format!("-I{}", tf_src_path.to_str().unwrap()))
        // Generate doc comments on bindings
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        .clang_arg("-fretain-comments-from-system-headers")
        // Use Rust enums across the board
        .rustified_enum("TfLiteFrameBufferFormat")
        .rustified_enum("TfLiteFrameBufferOrientation")
        .rustified_enum("TfLiteSupportErrorCode")
        .rustified_enum("CoreMLDelegateSettingsEnabledDevices")
        .rustified_enum("TfLiteCoralSettingsPerformance")
        // The layout tests are ugly
        .layout_tests(generate_bindgen_layout_tests())
        // Tell cargo to invalidate the built crate whenever any of the included header
        // files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_dir().join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn test_python_bin(python_bin_path: &str) -> bool {
    println!("Testing Python at {}", python_bin_path);
    let success = std::process::Command::new(python_bin_path)
        .args(&["-c", "import numpy, importlib.util"])
        .status()
        .map(|s| s.success())
        .unwrap_or_default();
    if success {
        println!("Using Python at {}", python_bin_path);
    }
    success
}

fn get_python_bin_path() -> Option<PathBuf> {
    if let Ok(val) = env::var("PYTHON_BIN_PATH") {
        if !test_python_bin(&val) {
            panic!("Given Python binary failed in test!")
        }
        Some(PathBuf::from(val))
    } else {
        let bin = if target_os() == "windows" {
            "where"
        } else {
            "which"
        };
        if let Ok(x) = std::process::Command::new(bin).arg("python3").output() {
            for path in String::from_utf8(x.stdout).unwrap().lines() {
                if test_python_bin(path) {
                    return Some(PathBuf::from(path));
                }
                println!("cargo:warning={:?} failed import test", path)
            }
        }
        if let Ok(x) = std::process::Command::new(bin).arg("python").output() {
            for path in String::from_utf8(x.stdout).unwrap().lines() {
                if test_python_bin(path) {
                    return Some(PathBuf::from(path));
                }
                println!("cargo:warning={:?} failed import test", path)
            }
            None
        } else {
            None
        }
    }
}

fn copy_or_overwrite<P: AsRef<Path> + Debug, Q: AsRef<Path> + Debug>(src: P, dest: Q) {
    let src_path: &Path = src.as_ref();
    let dest_path: &Path = dest.as_ref();
    if dest_path.exists() {
        if dest_path.is_file() {
            std::fs::remove_file(&dest_path).expect("Cannot remove file");
        } else {
            std::fs::remove_dir_all(&dest_path).expect("Cannot remove directory");
        }
    }
    if src_path.is_dir() {
        let options = fs_extra::dir::CopyOptions {
            copy_inside: true,
            ..fs_extra::dir::CopyOptions::new()
        };
        fs_extra::dir::copy(src_path, dest_path, &options).unwrap_or_else(|e| {
            panic!(
                "Cannot copy directory from {:?} to {:?}. Error: {}",
                src, dest, e
            )
        });
    } else {
        std::fs::copy(src_path, dest_path).unwrap_or_else(|e| {
            panic!(
                "Cannot copy file from {:?} to {:?}. Error: {}",
                src, dest, e
            )
        });
    }
}
