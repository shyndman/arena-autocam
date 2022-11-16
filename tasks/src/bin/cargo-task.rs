use anyhow::Result;
use cargo_task::{
    cargo::{workspace_path, RustBuildTargets},
    ctx::BuildContext,
    docker::{build_base_images, build_images_for_targets},
};
use clap::{Parser, Subcommand};
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

#[derive(Subcommand)]
enum Commands {
    /// Generates the base builder docker images
    BuildBaseImages,
    BuildImage(RustBuildTargets),
}

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        // cmd_lib prints out run_cmd! strings at debug
        .filter_level(LevelFilter::Debug)
        .init();
    cmd_lib::set_debug(true);

    let cli = Cli::parse();
    let repository_host_port = "ubuntu-desktop.local:5000".into();
    let repository_ip = lookup_host("ubuntu-desktop.local")?
        .first()
        .unwrap()
        .to_owned();
    let context = BuildContext::new(
        repository_host_port,
        repository_ip,
        workspace_path()?,
        cli.no_cache,
    );

    debug!("Build context created: {:#?}", context);

    match &cli.command {
        Commands::BuildBaseImages => build_base_images(&context),
        Commands::BuildImage(targets) => build_images_for_targets(targets, &context),
    }
}
