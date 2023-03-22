use anyhow::Result;

use super::manifest::{SyncedDocument, CadManifest};
use super::{environment_client, load_config};
use crate::ctx::TaskContext;

pub fn display_cad_info(task_ctx: &TaskContext) -> Result<()> {
    // Load the manifest describing what to sync
    let CadManifest {
        document: SyncedDocument {
            id: doc_id,
            workspace_id,
        },
        assemblies,
        ..
    } = load_config(task_ctx)?;

    let client = environment_client()?;
    let element_map = client.get_document_elements(&doc_id, &workspace_id)?;

    for sync_assembly in assemblies {
        if !element_map.contains_key(&sync_assembly.id) {
            panic!("Could not find an assembly ({})", sync_assembly.id);
        }

        let assembly = client.get_assembly(&doc_id, &workspace_id, &sync_assembly.id)?;
        for inst in assembly
            .all_part_instances()
            .filter(|(inst, _part)| !inst.is_standard_content)
        {
            println!("{:#?}", inst);
        }
    }

    Ok(())
}
