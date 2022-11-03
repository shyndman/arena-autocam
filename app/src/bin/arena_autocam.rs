use std::path::PathBuf;

use anyhow::Result;
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

fn main() -> Result<()> {
    eprintln!("\nStarting Arena Autocam");

    let args = Args::parse();
    let config = Config::new(args.config_path, args.config)?;

    eprintln!("Configuration:\n");
    eprintln!("{}", indent(config.to_toml_string()?.as_str(), "   "));
    eprintln!();

    match create_pipeline(&config)
        .and_then(|res| configure_pipeline(&config, res))
        .and_then(run_main_loop)
    {
        Ok(_r) => Ok(()),
        Err(e) => {
            eprintln!("Error! {}", e);
            Err(e)
        }
    }
}
