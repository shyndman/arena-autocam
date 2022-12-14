use std::net::IpAddr;

use cargo_metadata::camino::Utf8PathBuf;
use clap::Command;
use clap_complete::Shell;

#[derive(Debug)]
pub struct TaskContext {
    pub docker_repository_host_port: String,
    pub docker_repository_ip: IpAddr,
    pub workspace_root_path: Utf8PathBuf,
    pub command: Command,
    pub shell: Option<Shell>,
    pub no_cache: bool,
}

impl TaskContext {
    pub fn new(
        docker_repository_host_port: String,
        docker_repository_ip: IpAddr,
        workspace_root_path: Utf8PathBuf,
        command: Command,
        shell: Option<Shell>,
        no_cache: bool,
    ) -> Self {
        Self {
            docker_repository_host_port,
            docker_repository_ip,
            workspace_root_path,
            command,
            shell,
            no_cache,
        }
    }

    pub fn is_build_cache_enabled(&self) -> bool {
        !self.no_cache
    }
}
