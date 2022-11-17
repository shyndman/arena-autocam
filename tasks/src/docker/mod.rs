mod build;
mod run;

pub use build::*;
pub use run::*;

const IMAGE_APP_COMPONENT: &str = "arena-autocam";

/// Returns the fully qualified name of a Docker image in our private repository
pub fn qualified_image_name(image_basename: &String, registry_url: &String) -> String {
    format!(
        "{}/{}/{}",
        registry_url, IMAGE_APP_COMPONENT, image_basename
    )
}
