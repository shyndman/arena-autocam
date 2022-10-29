use arena_autocam::pipeline::{create_pipeline, run_main_loop};

fn main() {
    match create_pipeline().and_then(run_main_loop) {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }
}
