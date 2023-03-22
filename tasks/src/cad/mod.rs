mod display;
mod manifest;
#[allow(dead_code)]
mod models;
mod onshape_client;
mod pull;

use std::fs::{self, DirBuilder};

use anyhow::Result;
pub use display::display_cad_info;
use dotenv::dotenv;
pub use pull::pull_cad_files;

use self::manifest::CadManifest;
use self::onshape_client::OnShapeClient;
use crate::ctx::TaskContext;
const SYNC_MANIFEST_PATH: &str = "cad/manifest.toml";

pub(crate) fn environment_client() -> Result<OnShapeClient> {
    dotenv().ok();

    OnShapeClient::new(
        std::env::var("ONSHAPE_ACCESS_KEY")?,
        std::env::var("ONSHAPE_SECRET_KEY")?,
    )
}

pub(crate) fn load_config(task_ctx: &TaskContext) -> Result<CadManifest> {
    let sync_manifest_path = {
        let mut p = task_ctx.workspace_root_path.clone();
        p.push(SYNC_MANIFEST_PATH);
        p
    };
    let mut config: CadManifest = toml::from_str(&fs::read_to_string(sync_manifest_path)?)?;
    config.stl_root_path = {
        let mut p = task_ctx.workspace_root_path.clone();
        p.push("cad");
        p.push(config.stl_root_path);
        p.into()
    };
    DirBuilder::new()
        .recursive(true)
        .create(config.stl_root_path.clone())?;

    Ok(config)
}
