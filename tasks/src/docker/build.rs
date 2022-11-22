use std::collections::HashMap;

use anyhow::Result;
use cargo_metadata::camino::Utf8PathBuf;
use cmd_lib::run_cmd;

use crate::{
    cargo::{RustBuildProfile, RustBuildTarget, RustTargetId, TargetArchitecture},
    cmd::*,
    ctx::TaskContext,
    docker::qualified_image_name,
};

const BUILDER_IMAGE_BASENAME: &str = "builder_base";
const RUNNER_IMAGE_BASENAME: &str = "runner_base";
const BUILDER_NAME: &str = "arena-autocam_builder";

#[derive(Default)]
struct ImageBuildOptions {
    image_basename: String,
    dockerfile_path: Utf8PathBuf,
    docker_context_path: Utf8PathBuf,
    target_arch: Option<TargetArchitecture>,
    rust_build_target: Option<RustTargetId>,
    rust_profile: RustBuildProfile,
    additional_build_args: Option<HashMap<&'static str, String>>,
}

impl ImageBuildOptions {
    fn image_name_variants(&self, registry_url: &String) -> TargetImageNames {
        let qualified_image_name = qualified_image_name(&self.image_basename, registry_url);
        TargetImageNames {
            image_name: qualified_image_name.clone(),
            tagged_image_name: format!("{}:latest", qualified_image_name),
            build_cache_image_name: format!("{}:build-cache", qualified_image_name),
        }
    }
}

struct TargetImageNames {
    image_name: String,
    tagged_image_name: String,
    build_cache_image_name: String,
}

/// Builds the specified Docker image, and returns the image name as a [`String`].
fn build_image(options: ImageBuildOptions, task_ctx: &TaskContext) -> Result<String> {
    build_builder_if_required(None, task_ctx)?;

    let docker_repository_url = &task_ctx.docker_repository_host_port;
    let docker_repository_ip = &task_ctx.docker_repository_ip;

    let TargetImageNames {
        image_name: _image_name,
        tagged_image_name,
        build_cache_image_name,
    } = options.image_name_variants(&docker_repository_url);

    let dockerfile_path = options.dockerfile_path;
    let docker_context_path = options.docker_context_path;

    let rust_profile = options.rust_profile.to_string();
    let rust_profile_dir_args = options
        .rust_profile
        .output_dir_component()
        .into_docker_build_arg("RUST_PROFILE_OUT_DIR");
    let rust_build_target = options.rust_build_target.map_or(vec![], |name| {
        name.to_cargo_arg()
            .into_docker_build_arg("RUST_BUILD_TARGET")
    });
    let arch_derivative_args: Vec<String> = options.target_arch.map_or(vec![], |arch| {
        vec![
            pkg_config_build_arg_for_arch(arch),
            arch.rust_triple().into_docker_build_arg("RUST_TARGET"),
            arch.gcc_toolchain().into_docker_build_arg("GCC_TOOLCHAIN"),
            arch.to_string().into_docker_build_arg("DOCKER_TARGET_ARCH"),
            arch.docker_platform().into_cmd_arg("platform"),
        ]
        .concat()
    });

    let cache_args: Vec<String> = if task_ctx.is_build_cache_enabled() {
        vec![
            format!(
                "--cache-from=type=registry,ref={cache_name}",
                cache_name = build_cache_image_name
            ),
            format!(
                "--cache-to=type=registry,ref={cache_name},mode=max",
                cache_name = build_cache_image_name
            ),
        ]
    } else {
        vec!["--no-cache=true".into()]
    };
    let additional_build_args = options.additional_build_args.map_or(vec![], |build_args| {
        build_args
            .iter()
            .flat_map(|(k, v)| v.into_docker_build_arg(k))
            .collect()
    });

    // Build the image, and push it to the
    run_cmd! (
        docker --log-level=info buildx build
            --builder=$BUILDER_NAME
            --pull
            $[cache_args]
            --allow=network.host,security.insecure
            --network=host
            --add-host=ubuntu-desktop.local:$docker_repository_ip
            --progress=plain
            --file=$dockerfile_path
            // This is `--platform=linux/{arch}`, which may not be provided
            $[arch_derivative_args]
            --build-arg DOCKER_REPO=$docker_repository_url
            --build-arg RUST_PROFILE=$rust_profile
            $[rust_build_target]
            $[rust_profile_dir_args]
            $[additional_build_args]
            --tag=$tagged_image_name
            --output type=image,push=true
            $docker_context_path
    )?;

    run_cmd!(docker --log-level=info image pull $tagged_image_name)?;

    Ok(tagged_image_name)
}

fn pkg_config_build_arg_for_arch(arch: TargetArchitecture) -> Vec<String> {
    match arch {
        TargetArchitecture::Aarch64 => {
            let mut args = "PKG_CONFIG_ALLOW_CROSS=1"
                .to_string()
                .into_docker_build_arg("RUST_PKG_CONFIG_ALLOW_CROSS");
            args.extend(
                "PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig/"
                    .to_string()
                    .into_docker_build_arg("RUST_PKG_CONFIG_PATH"),
            );
            args
        }
        TargetArchitecture::Amd64 => vec![],
    }
}

pub fn build_image_for_target(
    target: &RustBuildTarget,
    task_ctx: &TaskContext,
) -> Result<String> {
    // Build the pre-build-io image, which is expensive, but almost always cached.
    build_image(
        ImageBuildOptions {
            image_basename: format!("pre_build_io_{}", target.arch.to_string()),
            dockerfile_path: "docker/2.pre-build-io.dockerfile".into(),
            docker_context_path: task_ctx.workspace_root_path.clone(),
            target_arch: Some(target.arch),
            ..Default::default()
        },
        task_ctx,
    )?;

    // Build the image that builds the binary
    let build_image_name = build_image(
        ImageBuildOptions {
            image_basename: target.builder_image_basename(),
            dockerfile_path: "docker/3.build-rust-target.dockerfile".into(),
            docker_context_path: task_ctx.workspace_root_path.clone(),
            target_arch: Some(target.arch),
            rust_profile: target.profile,
            rust_build_target: Some(target.id.clone()),
            ..Default::default()
        },
        task_ctx,
    )?;

    // Build the binary that runs the binary
    let mut additional_build_args = HashMap::default();
    additional_build_args.insert("DOCKER_BUILD_BIN_IMAGE", build_image_name);

    build_image(
        ImageBuildOptions {
            image_basename: target.runner_image_basename(),
            dockerfile_path: "docker/5.run-binary.dockerfile".into(),
            docker_context_path: task_ctx.workspace_root_path.clone(),
            target_arch: Some(target.arch),
            rust_profile: target.profile,
            additional_build_args: Some(additional_build_args),
            ..Default::default()
        },
        task_ctx,
    )
}

pub fn build_base_builder_images(task_ctx: &TaskContext) -> Result<()> {
    // Build the base build image
    build_image(
        ImageBuildOptions {
            image_basename: BUILDER_IMAGE_BASENAME.into(),
            dockerfile_path: "docker/0.builder-base.dockerfile".into(),
            docker_context_path: "docker".into(),
            ..Default::default()
        },
        task_ctx,
    )?;

    // Build the arch-specific build bases
    for arch in TargetArchitecture::values() {
        build_image(
            ImageBuildOptions {
                image_basename: format!("{}_{}", BUILDER_IMAGE_BASENAME, arch),
                dockerfile_path: format!("docker/1.builder-base.{}.dockerfile", arch).into(),
                docker_context_path: "docker".into(),
                target_arch: Some(arch),
                ..Default::default()
            },
            task_ctx,
        )?;
    }

    // Build the arch-specific runner bases
    for arch in TargetArchitecture::values() {
        let mut additional_build_args = HashMap::new();
        additional_build_args.insert(
            "DOCKER_BASE_RUN_IMAGE",
            match arch {
                TargetArchitecture::Amd64 => {
                    "balenalib/amd64-debian:bookworm-build".to_string()
                }
                TargetArchitecture::Aarch64 => {
                    "balenalib/raspberrypi4-64-debian:bookworm-run".to_string()
                }
            },
        );

        build_image(
            ImageBuildOptions {
                image_basename: format!("{}_{}", RUNNER_IMAGE_BASENAME, arch),
                dockerfile_path: "docker/4.runner-base.dockerfile".into(),
                docker_context_path: "docker".into(),
                target_arch: Some(arch),
                additional_build_args: Some(additional_build_args),
                ..Default::default()
            },
            task_ctx,
        )?;
    }

    Ok(())
}

pub fn build_base_runner_images(task_ctx: &TaskContext) -> Result<()> {
    // Build the arch-specific runner bases
    for arch in TargetArchitecture::values() {
        let mut additional_build_args = HashMap::new();
        additional_build_args.insert(
            "DOCKER_BASE_RUN_IMAGE",
            match arch {
                TargetArchitecture::Amd64 => {
                    "balenalib/amd64-debian:bookworm-build".to_string()
                }
                TargetArchitecture::Aarch64 => {
                    "balenalib/raspberrypi4-64-debian:bookworm-run".to_string()
                }
            },
        );

        build_image(
            ImageBuildOptions {
                image_basename: format!("{}_{}", RUNNER_IMAGE_BASENAME, arch),
                dockerfile_path: "docker/4.runner-base.dockerfile".into(),
                docker_context_path: "docker".into(),
                target_arch: Some(arch),
                additional_build_args: Some(additional_build_args),
                ..Default::default()
            },
            task_ctx,
        )?;
    }

    Ok(())
}

pub fn build_builder_if_required(force: Option<bool>, task_ctx: &TaskContext) -> Result<()> {
    // The caller can force the creation of a builder (which we implement by
    // destroying it first)
    if let Some(true) = force {
        run_cmd! (
            docker buildx rm $BUILDER_NAME
        )?;
    }
    let workspace_root_path = &task_ctx.workspace_root_path;

    // This command will fail if no builder exists with that name
    let builder_not_found = run_cmd!(docker buildx use $BUILDER_NAME).is_err();
    if builder_not_found {
        run_cmd!(
            docker buildx create --driver=docker-container
                --name=${BUILDER_NAME}
                --config $workspace_root_path/docker/config/buildkitd.toml
                --driver-opt=network=host
                --bootstrap --use
                --buildkitd-flags "--allow-insecure-entitlement security.insecure"
                $workspace_root_path
        )?;
    }

    Ok(())
}
