use std::path::PathBuf;

use anyhow::{Error, Result};
use cairo::Rectangle;
pub fn to_canonicalized_path_string(path: &PathBuf) -> Result<String> {
    path.canonicalize()?
        .to_str()
        .ok_or(anyhow::anyhow!("Could not convert path to str"))
        .and_then(|s| Result::Ok(s.to_string()))
}
