use anyhow::Result;
use cargo_metadata::camino::Utf8PathBuf;
use cmd_lib::run_cmd;

use crate::{
    cargo::{
        RustBuildProfile, RustBuildTarget, RustBuildTargets, RustTargetId, TargetArchitecture,
    },
    cmd::*,
    ctx::BuildContext,
};

const IMAGE_APP_COMPONENT: &str = "arena-autocam";
const BUILDER_NAME: &str = "arena-autocam_builder";

#[derive(Default)]
struct ImageBuildOptions {
    image_basename: String,
    dockerfile_path: Utf8PathBuf,
    docker_context_path: Utf8PathBuf,
    target_arch: Option<TargetArchitecture>,
    rust_build_target: Option<RustTargetId>,
    rust_profile: RustBuildProfile,
}

impl ImageBuildOptions {
    fn image_name_variants(&self, registry_url: &String) -> TargetImageNames {
        let qualified_image_name = format!(
            "{}/{}/{}",
            registry_url, IMAGE_APP_COMPONENT, self.image_basename
        );
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

fn build_image(options: ImageBuildOptions, context: &BuildContext) -> Result<()> {
    let docker_repository_url = &context.docker_repository_url;
    let docker_repository_ip = &context.docker_repository_ip;

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
    let rust_pkg_config_args = options
        .target_arch
        .map_or(vec![], |arch| pkg_config_build_arg_for_arch(arch));
    let rust_target_args = options.target_arch.map_or(vec![], |arch| {
        arch.rust_triple().into_docker_build_arg("RUST_TARGET")
    });
    let docker_target_arch_args = options.target_arch.map_or(vec![], |arch| {
        arch.to_string().into_docker_build_arg("DOCKER_TARGET_ARCH")
    });
    let docker_platform_args = options.target_arch.map_or(vec![], |arch| {
        arch.docker_platform().into_cmd_arg("platform")
    });
    let cache_args: Vec<String> = if context.is_build_cache_enabled() {
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
            $[docker_platform_args]
            --build-arg DOCKER_REPO=$docker_repository_url
            --build-arg RUST_PROFILE=$rust_profile
            $[rust_build_target]
            $[rust_target_args]
            $[rust_pkg_config_args]
            $[rust_profile_dir_args]
            $[docker_target_arch_args]
            --tag=$tagged_image_name
            --output type=image,push=true
            $docker_context_path
    )?;

    run_cmd!(docker --log-level=info image pull $tagged_image_name)?;

    Ok(())
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

fn build_image_for_target(target: RustBuildTarget, context: &BuildContext) -> Result<()> {
    build_builder_if_required(None, context)?;

    let dockerfile_path = [
        &context.workspace_root_path.as_str(),
        "docker",
        "3.build-single-target.dockerfile",
    ]
    .iter()
    .collect();
    build_image(
        ImageBuildOptions {
            image_basename: format!(
                "{}_build_{}",
                target.id.to_snake_name(),
                target.arch.to_string()
            ),
            dockerfile_path: dockerfile_path,
            docker_context_path: context.workspace_root_path.clone(),
            target_arch: Some(target.arch),
            rust_profile: target.profile,
            rust_build_target: Some(target.id),
        },
        context,
    )?;

    Ok(())
}

pub fn build_images_for_targets(
    targets: &RustBuildTargets,
    context: &BuildContext,
) -> Result<()> {
    // Build the pre-build-io image, which is expensive, but almost always cached.
    build_image(
        ImageBuildOptions {
            image_basename: format!("pre_build_io_{}", targets.arch.to_string()),
            dockerfile_path: "docker/2.pre-build-io.dockerfile".into(),
            docker_context_path: context.workspace_root_path.clone(),
            target_arch: Some(targets.arch),
            rust_profile: targets.profile,
            ..Default::default()
        },
        context,
    )?;

    for t in targets.into_iter() {
        build_image_for_target(t, context)?
    }

    Ok(())
}

pub fn build_base_images(context: &BuildContext) -> Result<()> {
    // Build the base build image
    let image_basename = "builder_base";
    build_image(
        ImageBuildOptions {
            image_basename: image_basename.into(),
            dockerfile_path: "docker/0.builder-base.dockerfile".into(),
            docker_context_path: "docker".into(),
            ..Default::default()
        },
        context,
    )?;

    // Build the arch-specific build bases
    for arch in TargetArchitecture::values() {
        build_image(
            ImageBuildOptions {
                image_basename: format!("{}_{}", image_basename, arch),
                dockerfile_path: format!("docker/1.builder-base.{}.dockerfile", arch).into(),
                docker_context_path: "docker".into(),
                target_arch: Some(arch),
                ..Default::default()
            },
            context,
        )?;
    }

    Ok(())
}

pub fn build_builder_if_required(force: Option<bool>, context: &BuildContext) -> Result<()> {
    // The caller can force the creation of a builder (which we implement by
    // destroying it first)
    if let Some(true) = force {
        run_cmd! (
            docker buildx rm $BUILDER_NAME
        )?;
    }
    let workspace_root_path = &context.workspace_root_path;

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
