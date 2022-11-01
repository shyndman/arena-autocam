use std::path::PathBuf;

use arena_autocam::{
    config::Config,
    pipeline::{configure_pipeline, create_pipeline, run_main_loop},
};
use clap::Parser;
use serde_derive::Serialize;
use textwrap::indent;

#[derive(Parser, Debug, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config_path: Option<PathBuf>,

    #[command(flatten)]
    pub config: Config,
}

fn main() -> anyhow::Result<()> {
    eprintln!("Starting Arena Autocam");

    let args = Args::parse();
    let config = Config::new(args.config_path, args.config)?;

    eprintln!("Configuration:\n");
    eprintln!("{}", indent(config.to_toml_string()?.as_str(), "   "));

    match create_pipeline()
        .and_then(|res| configure_pipeline(&config, res))
        .and_then(run_main_loop)
    {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("Error! {}", e);
            Err(e)
        }
    }
}
