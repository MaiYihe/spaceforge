mod config;
mod export;
mod logging;
mod placement_region_cache;
mod pipeline;
mod steps;

use logging::init_logging;
use pipeline::run_backend;

fn main() {
    init_logging();
    run();
}

fn run() {
    let config_path = std::env::var("ASSET_IMPORT_CONFIG")
        .unwrap_or_else(|_| "assets/config/asset_import.toml".to_string());
    run_backend(&config_path);
    // TODO: wire geometry_core search/layout execution here.
    println!("Running backend-only mode (no viewer).");
}
