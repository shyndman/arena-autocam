use anyhow::Result;

use super::{environment_client, load_config};
use crate::cad::manifest::{CadManifest, SyncedDocument};
use crate::ctx::TaskContext;

pub fn pull_cad_files(task_ctx: &TaskContext) -> Result<()> {
    // Load the manifest describing what to sync
    let CadManifest {
        stl_root_path: stl_directory_path,
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

        let part_instances = sync_assembly.synced_parts_map();
        let assembly = client.get_assembly(&doc_id, &workspace_id, &sync_assembly.id)?;
        for (inst, part) in assembly
            .all_part_instances()
            .filter(|(inst, _)| part_instances.contains_key(&inst.id))
        {
            let sync_info = part_instances
                .get(&inst.id)
                .expect("Missing synced part instance");
            let stl = client.get_part_stl(
                &part.document_id,
                &part.document_microversion,
                &part.element_id,
                &part.part_id,
                &inst.configuration,
            )?;

            let mut stl_path = stl_directory_path.clone();
            stl_path.push(sync_info.name.clone());
            stl_path.set_extension("stl");

            std::fs::write(stl_path, &stl)?;
        }
    }

    Ok(())
}
