use std::env;

/// Returns the arguments that were provided to the `docker run` command via the
/// `RUST_ARGS` environment variable.
pub fn dev_args() -> Vec<String> {
    if let Ok(var) = env::var("RUST_ARGS") {
        var.split_whitespace().map(|a| a.to_string()).collect()
    } else {
        vec![]
    }
}
