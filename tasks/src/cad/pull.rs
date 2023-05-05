use anyhow::Result;
use clap::Args;

use super::{environment_client, load_config};
use crate::cad::manifest::{CadManifest, SyncedDocument};
use crate::ctx::TaskContext;

#[derive(Args)]

pub struct PullCadFilesOptions {
    /// If true, the JSON OnShape assemblies will be written to stdout
    #[arg(long)]
    no_clean: bool,
}

pub fn pull_cad_files(options: &PullCadFilesOptions, task_ctx: &TaskContext) -> Result<()> {
    // Load the manifest describing what to sync
    let CadManifest {
        stl_root_path: stl_path,
        document: SyncedDocument {
            id: doc_id,
            workspace_id,
        },
        assemblies,
        ..
    } = load_config(task_ctx)?;

    if !options.no_clean {
        eprintln!("Cleaning {}", stl_path);

        let entries = std::fs::read_dir(&stl_path).expect("Could not read STL path");
        for res in entries {
            let entry = match res {
                Ok(entry) => entry,
                Err(e) => panic!("{}", e),
            };

            let name = entry.file_name().into_string().expect("");
            if name.ends_with(".stl") {
                std::fs::remove_file(entry.path()).expect("Could not delete file");
            }
        }
    }

    let client = environment_client()?;
    let element_map = client.get_document_elements(&doc_id, &workspace_id)?;

    for synced_assembly in assemblies {
        eprintln!(
            "ASSEMBLY \"{}\" ({})",
            synced_assembly.display_name, synced_assembly.id
        );

        if !element_map.contains_key(&synced_assembly.id) {
            panic!("Could not find an assembly ({})", synced_assembly.id);
        }

        let part_instances = synced_assembly.synced_parts_map();
        let assembly = client.get_assembly(&doc_id, &workspace_id, &synced_assembly.id)?;
        for (inst, part) in assembly
            .all_part_instances()
            .iter()
            .filter(|(inst, _)| part_instances.contains_key(&inst.id))
        {
            eprintln!("  PART {}...", inst.name);

            let synced_instance = part_instances
                .get(&inst.id)
                .expect("Missing synced part instance");
            let stl = client.get_part_stl(
                &part.document_id,
                &part.document_microversion,
                &part.element_id,
                &part.part_id,
                &inst.configuration,
            )?;

            let mut stl_path = stl_path.clone();
            stl_path.push(synced_instance.basename.clone());
            stl_path.set_extension("stl");

            std::fs::write(&stl_path, &stl)?;

            eprintln!("    written to {}", &stl_path);
        }
    }

    Ok(())
}
