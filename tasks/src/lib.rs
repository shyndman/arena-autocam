use cargo_metadata::camino::Utf8PathBuf;

pub mod cargo;
pub mod cmd;
pub mod docker;

pub struct BuildContext {
    pub docker_repository_url: String,
    pub workspace_root_path: Utf8PathBuf,
    pub no_cache: bool,
}

impl BuildContext {
    pub fn new(
        docker_repository_url: String,
        workspace_root_path: Utf8PathBuf,
        no_cache: bool,
    ) -> Self {
        Self {
            docker_repository_url,
            workspace_root_path,
            no_cache,
        }
    }
}
