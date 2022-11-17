use anyhow::Result;
use cargo_task::{
    cargo::{
        workspace_path, RustBuildProfile, RustBuildTarget, RustTargetId, TargetArchitecture,
    },
    ctx::TaskContext,
    docker::{
        build_base_builder_images, build_base_runner_images, build_image_for_target,
        run_image_for_targets,
    },
};
use clap::{ArgGroup, Args, Parser, Subcommand};
use dns_lookup::lookup_host;
use log::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long, default_value_t = false)]
    no_cache: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
struct BuildImageOptions {
    #[command(flatten)]
    target: RustBuildTargetOptions,
}

#[derive(Args)]
struct RunImageOptions {
    #[command(flatten)]
    build_target: RustBuildTargetOptions,
    /// Skip the build step and attempt to run immediately
    #[arg(long, default_value_t = false)]
    no_build: bool,
    /// Name of the context to use to connect to the daemon
    #[arg(long)]
    docker_context: Option<String>,
}

#[derive(Args)]
#[command(group(
    ArgGroup::new("target")
        .required(true)
        .args(["bin", "example"]),
))]
struct RustBuildTargetOptions {
    /// The name of the Rust binary target
    #[arg(long, default_value = None)]
    bin: Option<String>,
    /// The name of the Rust example target
    #[arg(long, default_value = None)]
    example: Option<String>,
    /// The Rust build profile
    #[arg(long, default_value = "dev")]
    profile: RustBuildProfile,
    /// The Rust build architecture
    #[arg(long, default_value = "amd64")]
    arch: TargetArchitecture,
}

impl RustBuildTargetOptions {
    fn to_rust_build_target(&self) -> RustBuildTarget {
        let target_id = if let Some(ref bin_name) = self.bin {
            RustTargetId::Bin(bin_name.clone())
        } else {
            RustTargetId::Example(self.example.as_ref().unwrap().clone())
        };
        RustBuildTarget {
            id: target_id,
            profile: self.profile,
            arch: self.arch,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Generates the base builder docker images
    BuildBaseBuilderImages,
    /// Generates the base runner docker images
    BuildBaseRunnerImages,
    BuildImage(BuildImageOptions),
    RunImage(RunImageOptions),
}

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Warn)
        // cmd_lib prints out run_cmd! command strings at debug, so that's where we set it
        .filter_module("cmd_lib::process", LevelFilter::Debug)
        .init();
    cmd_lib::set_debug(true);

    let cli = Cli::parse();
    // TODO(shyndman): Change this to pull from the config, environment, or args...
    // something other than a hardcode
    let repository_host_port = "ubuntu-desktop.local:5000".into();
    let repository_ip = lookup_host("ubuntu-desktop.local")?
        .first()
        .unwrap()
        .to_owned();
    let task_ctx = TaskContext::new(
        repository_host_port,
        repository_ip,
        workspace_path()?,
        cli.no_cache,
    );

    debug!("Build context created: {:#?}", task_ctx);

    match &cli.command {
        Commands::BuildBaseBuilderImages => build_base_builder_images(&task_ctx),
        Commands::BuildBaseRunnerImages => build_base_runner_images(&task_ctx),
        Commands::BuildImage(BuildImageOptions { target }) => {
            build_image_for_target(&target.to_rust_build_target(), &task_ctx).map(|_| ())
        }
        Commands::RunImage(RunImageOptions {
            build_target,
            docker_context,
            no_build,
        }) => run_image_for_targets(
            &build_target.to_rust_build_target(),
            docker_context.to_owned(),
            *no_build,
            &task_ctx,
        ),
    }
}
