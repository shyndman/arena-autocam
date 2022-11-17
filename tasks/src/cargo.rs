use std::fmt::Display;

use anyhow::Result;
use cargo_metadata::camino::Utf8PathBuf;
use clap::{Args, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RustBuildProfile {
    Bench,
    Dev,
    Release,
    Test,
}

impl RustBuildProfile {
    // The target directory is structured as follows:
    // `{project-root}/target/{target-triple}/{profile-dir}/`
    // where {profile-dir} is the value returned from this method.
    pub fn output_dir_component(&self) -> String {
        match self {
            RustBuildProfile::Bench => "release",
            RustBuildProfile::Dev => "debug",
            RustBuildProfile::Release => "release",
            RustBuildProfile::Test => "debug",
        }
        .to_string()
    }
}

impl Default for RustBuildProfile {
    fn default() -> Self {
        RustBuildProfile::Dev
    }
}

impl Display for RustBuildProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RustBuildProfile::Bench => "bench",
            RustBuildProfile::Dev => "dev",
            RustBuildProfile::Release => "release",
            RustBuildProfile::Test => "test",
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TargetArchitecture {
    Amd64,
    Aarch64,
}

impl TargetArchitecture {
    pub fn values() -> Vec<TargetArchitecture> {
        vec![Self::Amd64, Self::Aarch64]
    }

    pub fn rust_triple(&self) -> String {
        match self {
            TargetArchitecture::Amd64 => "x86_64-unknown-linux-gnu",
            TargetArchitecture::Aarch64 => "aarch64-unknown-linux-gnu",
        }
        .into()
    }

    pub fn docker_platform(&self) -> String {
        format!("linux/{}", self.to_string())
    }
}

impl Display for TargetArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TargetArchitecture::Amd64 => "amd64",
            TargetArchitecture::Aarch64 => "aarch64",
        })
    }
}

impl Default for TargetArchitecture {
    fn default() -> Self {
        TargetArchitecture::Amd64
    }
}

#[derive(Clone)]
pub enum RustTargetId {
    Bin(String),
    Example(String),
}
pub struct RustBuildTarget {
    pub id: RustTargetId,
    pub profile: RustBuildProfile,
    pub arch: TargetArchitecture,
}
impl RustTargetId {
    pub fn to_cargo_arg(&self) -> String {
        match self {
            RustTargetId::Bin(name) => format!("--bin={}", name),
            RustTargetId::Example(name) => format!("--example={}", name),
        }
    }

    pub fn to_snake_name(&self) -> String {
        match self {
            RustTargetId::Bin(name) => format!("{}_bin", name),
            RustTargetId::Example(name) => format!("{}_example", name),
        }
    }
}

#[derive(Args)]
pub struct RustBuildTargets {
    #[arg(long, default_value = None)]
    pub bin: Option<String>,
    #[arg(long, default_value = None)]
    pub example: Option<String>,
    #[arg(long, default_value = "dev")]
    pub profile: RustBuildProfile,
    #[arg(long, default_value = "amd64")]
    pub arch: TargetArchitecture,
}

impl IntoIterator for &RustBuildTargets {
    type Item = RustBuildTarget;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut v: Vec<Self::Item> = vec![];

        if let Some(ref bin_name) = self.bin {
            v.push(RustBuildTarget {
                id: RustTargetId::Bin(bin_name.to_string()),
                arch: self.arch,
                profile: self.profile,
            });
        }
        if let Some(ref example_name) = self.example {
            v.push(RustBuildTarget {
                id: RustTargetId::Example(example_name.to_string()),
                arch: self.arch,
                profile: self.profile,
            });
        }

        v.into_iter()
    }
}

pub fn workspace_path() -> Result<Utf8PathBuf> {
    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd.exec()?;
    Ok(metadata.workspace_root)
}
