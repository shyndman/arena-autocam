use std::collections::HashMap;

use cargo_metadata::camino::Utf8PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CadManifest {
    pub parasolid_root_path: Utf8PathBuf,
    pub stl_root_path: Utf8PathBuf,
    pub document: SyncedDocument,
    pub assemblies: Vec<SyncedAssembly>,
}

#[derive(Debug, Deserialize)]
pub struct SyncedDocument {
    pub id: String,
    pub workspace_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncedAssembly {
    pub display_name: String,
    pub id: String,
    pub synced_parts: Vec<SyncedPartInstance>,
}
impl SyncedAssembly {
    pub(crate) fn synced_parts_map(&self) -> HashMap<String, &SyncedPartInstance> {
        self.synced_parts
            .iter()
            .map(|i| (i.id.clone(), i))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncedPartInstance {
    pub id: String,
    pub basename: String,
}
