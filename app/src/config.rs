use std::path::PathBuf;

use anyhow::*;
use chrono::{DateTime, Duration, Local};
use clap::{Args, ArgGroup};
use figment::providers::{Format, Serialized, Toml};
use figment::value::magic::RelativePathBuf;
use figment::Figment;
use gst::prelude::TimeFormatConstructor;
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use strfmt::strfmt;

use crate::logging::*;

pub(self) static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_CONFIG",
        gst::DebugColorFlags::FG_YELLOW,
        Some("Auto-Arena Configuration"),
    )
});

// The application's configuration.
#[derive(Args, Debug, Deserialize, Serialize)]
pub struct Config {
    #[command(flatten)]
    pub source: SourceConfig,

    #[command(flatten)]
    pub detection: DetectionConfig,

    #[command(flatten)]
    pub video_storage: VideoStorageConfig,
}

impl Validate for Config {
    fn validate(&self) -> Result<&Self> {
        self.detection.validate()?;
        self.video_storage.validate()?;
        Ok(&self)
    }
}

#[derive(Args, Debug, Deserialize, Serialize)]
pub struct SourceConfig {
    /// The preferred width of the record stream (in pixels)
    #[arg(long, default_value_t = 1280)]
    pub record_stream_width: i32,

    /// The preferred height of the record stream (in pixels)
    #[arg(long, default_value_t = 720)]
    pub record_stream_height: i32,

    /// The width of the inference stream (in pixels). This should be the exact size the
    /// TFLite model requires, to avoid unnecessary scaling transformations.
    #[arg(long, default_value_t = 224)]
    pub infer_stream_width: i32,

    /// The height of the inference stream (in pixels). This should be the exact size the
    /// TFLite model requires, to avoid unnecessary scaling transformations.
    #[arg(long, default_value_t = 224)]
    pub infer_stream_height: i32,

    /// If provided, the pipeline will operate on a video at the provided path, rather
    /// than the camera stream.
    ///
    /// This is primarily for debugging
    #[arg(long, value_name = "FILE")]
    pub debug_source_video_path: Option<String>,
}

impl Validate for SourceConfig {
    fn validate(&self) -> Result<&Self> {
        match self.debug_source_video_path {
            Some(ref path) => {
                if PathBuf::from(path).is_file() {
                    Ok(self)
                } else {
                    Err(anyhow!("debug_source_model_path: file not found"))
                }
            }
            _ => Ok(self),
        }
    }
}

/// Configures the pipeline's target detection.
///
/// In general, this configures Tensorflow Lite, but it can also be configured to use
/// a color detection algorithm for debugging with less complexity.
#[derive(Args, Debug, Deserialize, Serialize)]
#[command(group(
    ArgGroup::new("debug_color_detection_group")
        .args(["color_detection_pixel_threshold"])
        .requires("debug_use_color_detection")
))]
pub struct DetectionConfig {
    /// The path to the Tensorflow Lite model
    #[serde(serialize_with = "RelativePathBuf::serialize_relative")]
    #[arg(long, value_name = "FILE", default_value = "./")]
    pub model_path: RelativePathBuf,

    /// The maximum number of results returned by the model per inference run.
    #[arg(long, default_value_t = 6)]
    pub max_results: u32,

    /// The score threshold under which a potential result is considered unimportant.
    #[arg(long, default_value_t = 0.2)]
    pub score_threshold: f32,

    #[arg(long, default_value_t = 5.0)]
    pub rate_per_second: f32,

    /// If true, the pipeline will be configured to use a green color detection algorithm
    /// instead of the horse detection ML.
    #[arg(long, default_value_t = false)]
    pub debug_use_color_detection: bool,

    /// Groups of colored pixels with more than this number will be considered a possible
    /// detection, if `--debug-use-color-detection` is enabled.
    #[arg(long, default_value_t = 10)]
    pub color_detection_pixel_threshold: u32,
}

impl DetectionConfig {
    pub fn inference_frame_duration(&self) -> Duration {
        Duration::milliseconds((1000.0 / self.rate_per_second) as i64)
    }

    /// `true` if the application is configured to use machine learning
    pub fn is_ml(&self) -> bool {
        !self.debug_use_color_detection
    }
}

impl Validate for DetectionConfig {
    fn validate(&self) -> Result<&Self> {
        if !self.debug_use_color_detection && !self.model_path.relative().is_file() {
            return Err(anyhow!(r"inference.model_path: file not found"));
        }
        return Ok(self);
    }
}

#[derive(Args, Debug, Deserialize, Serialize)]
pub struct VideoStorageConfig {
    /// The path where videos are stored locally before uploading.
    #[serde(serialize_with = "RelativePathBuf::serialize_relative")]
    #[arg(long, value_name = "DIR", default_value = "./")]
    pub temp_dir_path: RelativePathBuf,

    /// A formatting string describing how temporary video files are named.
    ///
    /// The following template variables are available, and need to appear in the string:
    ///
    ///     {session_datetime}
    ///     {chunk_number}
    ///
    /// The file extension will be added by the application.
    #[arg(long, default_value = "session-{session_datetime}-{chunk_number}")]
    pub video_filename_basename: String,

    /// The number of seconds of video written to a chunk before creating a new one.
    #[arg(long, default_value_t = 240)]
    pub video_chunk_duration_secs: u64,
}

impl VideoStorageConfig {
    pub fn video_chunk_duration_nanos(&self) -> u64 {
        self.video_chunk_duration_secs.seconds().nseconds()
    }

    pub fn video_path_pattern_for_datetime(&self, ts: DateTime<Local>) -> Result<String> {
        let mut path = self.temp_dir_path.relative().canonicalize()?;
        let basename = strfmt!(self.video_filename_basename.as_str(),
            session_datetime => ts.format("%Y%m%dT%H%M%S").to_string(),
            chunk_number => "%04d")?;
        path.push(basename);
        path.set_extension("mp4");
        Ok(path
            .to_str()
            .expect("Conversion between path and string failed")
            .to_string())
    }
}

impl Validate for VideoStorageConfig {
    fn validate(&self) -> Result<&Self> {
        if !self.temp_dir_path.relative().is_dir() {
            return Err(Error::msg(
                r"video_storage.temp_dir_path: directory not found",
            ));
        }

        if !self.video_filename_basename.contains("{session_datetime}") {
            return Err(Error::msg(
                r"video_storage.video_filename_basename: {session_datetime} missing",
            ));
        } else if !self.video_filename_basename.contains("{chunk_number}") {
            return Err(Error::msg(
                r"video_storage.video_filename_basename: {chunk_number} missing",
            ));
        }

        if self.video_chunk_duration_secs < 10 {
            return Err(Error::msg(
                r"video_storage.video_chunk_duration_secs must be >=10 seconds",
            ));
        }

        Ok(self)
    }
}

impl Config {
    pub fn new(
        user_config_path: Option<PathBuf>,
        cli_and_defaults_config: Config,
    ) -> Result<Self> {
        info!(CAT, "Loading configuration");

        let mut cfg = Figment::from(Serialized::defaults(cli_and_defaults_config));
        if let Some(path) = user_config_path {
            cfg = cfg.merge(Toml::file(path));
        };
        cfg.extract_validated().map_err(|e| anyhow!(e))
    }

    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(&self).map_err(|e| anyhow!(e))
    }
}

trait Validate: Sized {
    fn validate(&self) -> Result<&Self>;
}

trait ValidatedFigment {
    fn extract_validated<'a, T: serde::Deserialize<'a> + Validate>(&self) -> Result<T>;
}

impl ValidatedFigment for Figment {
    fn extract_validated<'a, T: serde::Deserialize<'a> + Validate>(&self) -> Result<T> {
        let value: T = self.extract()?;
        if let Err(err) = (&value).validate() {
            Err(err)
        } else {
            Ok(value)
        }
    }
}
