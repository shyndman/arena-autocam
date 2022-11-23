use std::{env, fs, os::unix::prelude::PermissionsExt};

use anyhow::Result;
use cmd_lib::run_cmd;
use log::info;

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

    let mut docker_run_cmd = std::process::Command::new("docker");
    let child = docker_run_cmd
        .arg("--log-level=info")
        .args(docker_context_arg.into_iter())
        .arg("run")
        .args([
            // RUST_LOG
            "--env",
            format!(
                "RUST_LOG={}",
                env::var("RUST_LOG")
                    .map(|s| s.to_string())
                    .unwrap_or("info".into())
            )
            .as_str(),
            // Privileged is necessary so that the host's devices are accessible
            "--privileged",
            // Use host networking, so that the private docker repository is accessible
            "--network=host",
            // Required by the balena base image
            "--tty",
            // Ensures that the process can receive SIGTERM signals
            "--init",
            // Remove the container after completion
            "--rm",
            // Always check for newer images
            "--pull=always",
            format!("{}:latest", image_name).as_str(),
        ])
        .spawn();

    let child_id = child.as_ref().ok().map(|c| c.id());
    ctrlc::set_handler(move || {
        if let Some(child_id) = child_id {
            run_cmd!(kill $child_id).ok();
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Wait for run to finish, immediately fixing dev permissions if the container
    // mangled them.
    let wait_res = child?.wait();
    ensure_correct_dev_permissions()?;

    wait_res?;
    Ok(())
}

pub fn ensure_correct_dev_permissions() -> Result<()> {
    info!("Restoring ptmx permissions if necessary");

    // TODO(shyndman): Are there cases where `docker run` will fail AFTER the permission
    // change has taken place?
    let mut perms = fs::metadata(PTMX_PATH)?.permissions();
    perms.set_mode(0o666);
    Ok(())
}
