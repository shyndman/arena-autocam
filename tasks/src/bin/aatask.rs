use aa_task::cad::{display_cad_info, pull_cad_files};
use aa_task::cargo::{
    workspace_path, RustBuildProfile, RustBuildTarget, RustTargetId, TargetArchitecture,
};
use aa_task::cli::{generate_completion_script, get_current_shell};
use aa_task::ctx::TaskContext;
use aa_task::docker::{
    build_base_builder_images, build_base_runner_images, build_image_for_target,
    run_image_for_targets,
};
use anyhow::Result;
use clap::error::ErrorKind;
use clap::{ArgGroup, Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use dns_lookup::lookup_host;
use log::*;

#[derive(Parser)]
#[command(name="task", author, version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[arg(long, default_value_t = false)]
    no_cache: bool,

    #[arg(long, default_value_t = false)]
    quiet: bool,

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

#[derive(Args)]

struct GenerateCompletionScriptOptions {
    /// The name of the shell for which completions will be generated.
    ///
    /// If not provided, aatask will attempt to detect the type of the invoking shell.
    shell: Option<Shell>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates the base builder docker images
    BuildBaseBuilderImages,
    /// Generates the base runner docker images
    BuildBaseRunnerImages,
    /// Builds a containerized Rust binary
    BuildImage(BuildImageOptions),
    /// Runs a containerized Rust binary
    RunImage(RunImageOptions),
    /// Generates a completion script for this utility
    GenerateCompletionScript(GenerateCompletionScriptOptions),
    /// Displays information about the CAD parts found in the OnShape assemblies identified
    /// in `/cad/manifest.toml`
    DisplayRemoteCadInfo,
    /// Pulls the latest CAD files (STL, FeatureScript) from OnShape, and write them to the
    /// `/cad` directory
    PullCadFiles,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let log_level = if cli.quiet {
        log::Level::Warn
    } else {
        log::Level::Debug
    };
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log_level.to_level_filter())
        // cmd_lib prints out run_cmd! command strings at debug, so that's where we set it
        .filter_module("cmd_lib::process", LevelFilter::Debug)
        .filter_module("reqwest", LevelFilter::Info)
        .init();
    cmd_lib::set_debug(true);

    // TODO(shyndman): Change this to pull from the config, environment, or args...
    // something other than a hardcode
    let repository_host_port = "ubuntu-desktop.local:5000".into();
    let repository_ip = lookup_host("ubuntu-desktop.local")?
        .first()
        .unwrap()
        .to_owned();

    let mut task_ctx = TaskContext::new(
        repository_host_port,
        repository_ip,
        workspace_path()?,
        Cli::command(),
        get_current_shell()?,
        cli.no_cache,
    );

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
        Commands::GenerateCompletionScript(GenerateCompletionScriptOptions {
            shell: shell_arg,
        }) => {
            let Some(target_shell) = shell_arg.or(task_ctx.shell) else {
                let err: clap::Error = task_ctx.command.error(
                    ErrorKind::MissingRequiredArgument,
                    "No invoking shell found. Please provide a SHELL arg"
                );
                err.exit();
            };

            let script = generate_completion_script(target_shell, &mut task_ctx)?;
            println!("{}", script);

            Ok(())
        }
        Commands::DisplayRemoteCadInfo => display_cad_info(&task_ctx),
        Commands::PullCadFiles => pull_cad_files(&task_ctx),
    }
}
