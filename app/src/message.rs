extern crate derive_more;

use std::time::Duration;

use anyhow::{anyhow, Result};
use gst::ClockTime;
use lazy_static::lazy_static;
use regex::Regex;
use strum_macros::{Display as DisplayEnum, EnumString};

use crate::foundation::geom::Rect;

const APP: &'static str = "aa";

lazy_static! {
    static ref KIND_PATTERN: Regex = {
        let s = format!(r"^\w*(?P<app>{})/(?P<name>[\w\d_\.-]+)\w*$", APP);
        Regex::new(&s).unwrap()
    };
}

#[derive(Debug, DisplayEnum, EnumString)]
#[strum(serialize_all = "kebab-case")]
#[strum(ascii_case_insensitive)]
pub enum AAMessage {
    /// Emitted when the inference engine begins operating upon a frame.
    ///
    /// `dts` can be considered this frame's identifier, and will be the same across
    /// `InferFrameStart`, `InferObjectDetection` and `InferFrameDone` events on the same
    /// frame.
    InferFrameStart { dts: ClockTime },
    /// Indicates that an object detection has taken place. One message will be emitted
    /// on the bus per object.
    InferObjectDetection(DetectionDetails),
    InferFrameDone {
        dts: ClockTime,
        duration: Duration,
        detection_count: i32,
    },
}

impl AAMessage {
    pub fn kind_matches(str: &str) -> bool {
        KIND_PATTERN.is_match(str)
    }

    pub fn kind(&self) -> String {
        format!("{}/{}", APP, self)
    }

    pub fn from_gst_message(msg: &gst::Message) -> Result<Self> {
        let structure = msg
            .structure()
            .ok_or(anyhow!("No message structure found"))?;

        Self::from_gst_message_structure(structure)
    }

    pub fn from_gst_message_structure(structure: &gst::StructureRef) -> Result<Self> {
        if !Self::kind_matches(structure.name()) {
            return Err(anyhow!("Provided structure is not an AAMessage"));
        }

        let message_name = structure.name().split("/").last().unwrap();

        // We first deserialize into an empty enum variant, then match on it, and from
        // that determine what we should pull out of the provided structure.
        let empty_message: AAMessage = message_name.parse()?;
        let full_message = match empty_message {
            AAMessage::InferFrameStart { .. } => AAMessage::InferFrameStart {
                dts: structure.get("dts")?,
            },
            AAMessage::InferObjectDetection(..) => {
                AAMessage::InferObjectDetection(DetectionDetails {
                    pts: structure.get("dts")?,
                    label: structure.get("label")?,
                    score: structure.get("score")?,
                    bounds: structure.get("bounds")?,
                })
            }
            AAMessage::InferFrameDone { .. } => AAMessage::InferFrameDone {
                dts: structure.get("dts")?,
                detection_count: structure.get("detection_count")?,
                duration: structure.get::<ClockTime>("duration")?.into(),
            },
        };
        Ok(full_message)
    }

    pub fn to_gst_message(&self) -> Result<gst::Message> {
        let mut structure = gst::Structure::new_empty(&self.kind());
        match self {
            AAMessage::InferFrameStart { dts } => {
                structure.set("dts", dts);
            }
            AAMessage::InferObjectDetection(DetectionDetails {
                pts: dts,
                label,
                score,
                bounds,
            }) => {
                structure.set("dts", dts);
                structure.set("label", label);
                structure.set("score", score);
                structure.set("bounds", bounds);
            }
            AAMessage::InferFrameDone {
                dts,
                detection_count,
                duration,
            } => {
                structure.set("dts", dts);
                structure.set("detection_count", detection_count);
                structure.set(
                    "duration",
                    <ClockTime as TryFrom<Duration>>::try_from(*duration).unwrap(),
                );
            }
        }
        Ok(gst::message::Application::builder(structure).build())
    }
}

#[derive(Clone, Debug, Default)]
pub struct DetectionDetails {
    pub pts: ClockTime,
    pub label: String,
    pub score: f32,
    pub bounds: Rect,
}
