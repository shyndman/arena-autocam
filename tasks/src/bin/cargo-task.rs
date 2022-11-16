use anyhow::Result;
use cargo_task::{
    cargo::{workspace_path, RustBuildTargets},
    docker::{build_base_images, build_images_for_targets},
    BuildContext,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long, default_value_t = false)]
    no_cache: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates the base builder docker images
    BuildBaseImages,
    BuildImage(RustBuildTargets),
}

fn main() -> Result<()> {
    cmd_lib::init_builtin_logger();

    let cli = Cli::parse();
    let context = BuildContext::new(
        "ubuntu-desktop.local:5000".into(),
        workspace_path()?,
        cli.no_cache,
    );

    match &cli.command {
        Commands::BuildBaseImages => build_base_images(&context),
        Commands::BuildImage(targets) => build_images_for_targets(targets, &context),
    }
}
