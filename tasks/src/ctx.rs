use std::net::IpAddr;

use cargo_metadata::camino::Utf8PathBuf;

#[derive(Debug)]
pub struct BuildContext {
    pub docker_repository_url: String,
    pub docker_repository_ip: IpAddr,
    pub workspace_root_path: Utf8PathBuf,
    pub no_cache: bool,
}

impl BuildContext {
    pub fn new(
        docker_repository_url: String,
        docker_repository_ip: IpAddr,
        workspace_root_path: Utf8PathBuf,
        no_cache: bool,
    ) -> Self {
        Self {
            docker_repository_url,
            docker_repository_ip,
            workspace_root_path,
            no_cache,
        }
    }

    pub fn is_build_cache_enabled(&self) -> bool {
        !self.no_cache
    }
}
