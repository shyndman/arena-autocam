use std::net::IpAddr;

use cargo_metadata::camino::Utf8PathBuf;

#[derive(Debug)]
pub struct TaskContext {
    pub docker_repository_host_port: String,
    pub docker_repository_ip: IpAddr,
    pub workspace_root_path: Utf8PathBuf,
    pub no_cache: bool,
}

impl TaskContext {
    pub fn new(
        docker_repository_host_port: String,
        docker_repository_ip: IpAddr,
        workspace_root_path: Utf8PathBuf,
        no_cache: bool,
    ) -> Self {
        Self {
            docker_repository_host_port,
            docker_repository_ip,
            workspace_root_path,
            no_cache,
        }
    }

    pub fn is_build_cache_enabled(&self) -> bool {
        !self.no_cache
    }
}
