use std::{fs, os::unix::prelude::PermissionsExt};

use anyhow::Result;
use cmd_lib::run_cmd;

use crate::{
    cargo::RustBuildTarget,
    cmd::IntoCliArgsVec,
    ctx::TaskContext,
    docker::{build_image_for_target, qualified_image_name},
};

/// The Balena images we use expose system devices to the Docker container, but oddly,
/// will change the permissions on this device. On some OSes (Ubuntu is the only known)
/// this prevents new terminals from being opened until the permissions are reset to
/// 0666.
///
/// Related bug: https://github.com/balena-io-library/base-images/issues/597
const PTMX_PATH: &str = "/dev/pts/ptmx";

/// Run, and optionally build, a containerized Rust binary.
pub fn run_image_for_targets(
    target: &RustBuildTarget,
    docker_ctx: Option<String>,
    no_build: bool,
    task_ctx: &TaskContext,
) -> Result<()> {
    // Build the image we're going to run unless instructed otherwise
    if !no_build {
        build_image_for_target(target, task_ctx)?;
    }

    // Run it!
    let image_name = qualified_image_name(
        &target.runner_image_basename(),
        &task_ctx.docker_repository_host_port,
    );
    let docker_context_arg = docker_ctx.map_or(vec![], |ctx| ctx.into_cmd_arg("context"));

    // Ensure we correct dev permissions on an interrupt
    ctrlc::set_handler(move || {
        ensure_correct_dev_permissions().unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // Build the image, and push it to the
    run_cmd! (
        docker
            --log-level=info
            $[docker_context_arg]
        run
            --privileged
            --network=host
            --tty
            --pull=always
            $image_name:latest
    )?;
    ensure_correct_dev_permissions()?;

    Ok(())
}

pub fn ensure_correct_dev_permissions() -> Result<()> {
    // TODO(shyndman): Are there cases where `docker run` will fail AFTER the permission
    // change has taken place?
    let mut perms = fs::metadata(PTMX_PATH)?.permissions();
    perms.set_mode(0o666);
    Ok(())
}
