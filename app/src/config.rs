use std::path::PathBuf;

use anyhow::*;
use chrono::{DateTime, Local};
use clap::Args;
use figment::{
    providers::{Format, Serialized, Toml},
    value::magic::RelativePathBuf,
    Figment,
};
use gst::prelude::TimeFormatConstructor;
use serde::Deserialize as Deserde;
use serde_derive::{Deserialize, Serialize};
use strfmt::strfmt;

// The application's configuration.
#[derive(Args, Debug, Deserialize, Serialize)]
pub struct Config {
    #[command(flatten)]
    pub inference: InferenceConfig,
    #[command(flatten)]
    pub video_storage: VideoStorageConfig,
}

impl Validate for Config {
    fn validate(&self) -> Result<&Self> {
        self.inference.validate()?;
        self.video_storage.validate()?;
        Ok(&self)
    }
}

#[derive(Args, Debug, Deserialize, Serialize)]
pub struct InferenceConfig {
    /// The path to the Tensorflow Lite model
    #[serde(serialize_with = "RelativePathBuf::serialize_relative")]
    #[arg(long, value_name = "FILE", default_value = "./")]
    pub model_path: RelativePathBuf,

    /// The maximum number of results returned by the model per inference run.
    #[arg(long, default_value_t = 3)]
    pub max_results: u32,

    /// The score threshold under which a potential result is considered unimportant.
    #[arg(long, default_value_t = 0.1)]
    pub score_threshold: f32,
}

impl Validate for InferenceConfig {
    fn validate(&self) -> Result<&Self> {
        if !self.model_path.relative().is_file() {
            return Err(Error::msg(r"inference.model_path: file not found"));
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
    fn extract_validated<'a, T: Deserde<'a> + Validate>(&self) -> Result<T>;
}

impl ValidatedFigment for Figment {
    fn extract_validated<'a, T: Deserde<'a> + Validate>(&self) -> Result<T> {
        let value: T = self.extract()?;
        if let Err(err) = (&value).validate() {
            Err(err)
        } else {
            Ok(value)
        }
    }
}
