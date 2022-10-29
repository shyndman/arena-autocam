extern crate derive_more;

use anyhow::Result;
use cairo::Rectangle;
use gst::ClockTime;
use lazy_static::lazy_static;
use regex::Regex;
use strum_macros::{Display as DisplayEnum, EnumString};

const APP: &'static str = "arena_autocam";

lazy_static! {
    static ref MSG_TYPE_PATTERN: Regex = {
        let s = format!(
            r"^\w*(?P<app>{})/(?P<cat>[\w\d_\.-]+)/(?P<name>[\w\d_\.-]+)\w*$",
            APP
        );
        Regex::new(&s).unwrap()
    };
}

#[derive(Debug, DisplayEnum, EnumString, PartialEq, PartialOrd)]
#[strum(serialize_all = "kebab-case")]
#[strum(ascii_case_insensitive)]
pub enum Category {
    Detection,
}

#[derive(Debug)]
pub struct AppMsgType {
    pub application: &'static str,
    pub category: Category,
    pub name: String,
}

impl std::fmt::Display for AppMsgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "{}/{}/{}",
                self.application,
                self.category.to_string(),
                self.name
            )
            .as_str(),
        )?;
        Ok(())
    }
}

impl TryFrom<String> for AppMsgType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        let c = MSG_TYPE_PATTERN.captures(&value);
        if let Some(c) = c {
            Ok(Self {
                application: APP,
                category: Category::try_from(c.name("cat").unwrap().as_str())?,
                name: c.name("name").unwrap().as_str().into(),
            })
        } else {
            Err(anyhow::Error::msg("Did not match type pattern"))
        }
    }
}

pub trait DetectionMsg {
    fn msg_type(&self) -> AppMsgType {
        AppMsgType {
            application: APP,
            category: self.category(),
            name: self.name(),
        }
    }

    fn msg_type_as_string(&self) -> String {
        self.msg_type().to_string()
    }

    fn category(&self) -> Category {
        Category::Detection
    }

    fn name(&self) -> String;

    fn to_structure(&self) -> gst::Structure {
        let s = gst::Structure::new_empty(self.msg_type_as_string().as_str());
        self.to_structure_internal(s)
    }

    fn to_structure_internal(&self, structure: gst::Structure) -> gst::Structure;
}

#[derive(Debug)]
pub struct ObjectDetection {
    pub dts: ClockTime,
    pub label: String,
    pub score: f32,
    pub bounds: Rectangle,
}

impl DetectionMsg for ObjectDetection {
    fn name(&self) -> String {
        "object-detection".into()
    }

    fn to_structure_internal(&self, mut structure: gst::Structure) -> gst::Structure {
        structure.set("dts", &self.dts);
        structure.set("label", &self.label);
        structure.set("score", &self.score);
        structure.set("bounds", &self.bounds);
        structure
    }
}

impl TryFrom<&gst::StructureRef> for ObjectDetection {
    type Error = anyhow::Error;

    fn try_from(msg_structure: &gst::StructureRef) -> Result<Self> {
        Ok(Self {
            dts: msg_structure.get("dts")?,
            label: msg_structure.get("label")?,
            score: msg_structure.get("score")?,
            bounds: msg_structure.get("bounds")?,
        })
    }
}

pub struct DetectionFrameStart {
    pub dts: ClockTime,
}

impl DetectionMsg for DetectionFrameStart {
    fn name(&self) -> String {
        "detection-frame-start".into()
    }

    fn to_structure_internal(&self, mut structure: gst::Structure) -> gst::Structure {
        structure.set("dts", self.dts);
        structure
    }
}

impl TryFrom<&gst::StructureRef> for DetectionFrameStart {
    type Error = anyhow::Error;

    fn try_from(msg_structure: &gst::StructureRef) -> Result<Self> {
        Ok(Self {
            dts: msg_structure.get("dts")?,
        })
    }
}

pub struct DetectionFrameDone {
    pub dts: ClockTime,
    pub detection_count: i32,
}

impl DetectionMsg for DetectionFrameDone {
    fn name(&self) -> String {
        "detection-frame-done".into()
    }

    fn to_structure_internal(&self, mut structure: gst::Structure) -> gst::Structure {
        structure.set("dts", self.dts);
        structure.set("detection_count", self.detection_count);
        structure
    }
}

impl TryFrom<&gst::StructureRef> for DetectionFrameDone {
    type Error = anyhow::Error;

    fn try_from(msg_structure: &gst::StructureRef) -> Result<Self> {
        Ok(Self {
            dts: msg_structure.get("dts")?,
            detection_count: msg_structure.get("detection_count")?,
        })
    }
}

#[test]
fn main() {
    AppMsgType::try_from("foo/bar/baz".to_string()).expect_err("nope");
    AppMsgType::try_from(format!("{APP}/non-existent/baz").to_string()).expect_err("nope");
    AppMsgType::try_from(format!("{APP}/detection/baz").to_string()).expect("yup");
    assert_eq!(
        AppMsgType::try_from(format!("{APP}/detection/baz").to_string())
            .unwrap()
            .to_string(),
        "arena_autocam/detection/baz"
    );
}
