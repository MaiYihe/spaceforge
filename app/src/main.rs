fn main() {
    #[cfg(feature = "viewer")]
    {
        let config = load_viewer_config();
        viewer::run_with_config(config);
        return;
    }

    #[cfg(not(feature = "viewer"))]
    {
        run_backend();
    }
}

#[cfg(not(feature = "viewer"))]
fn run_backend() {
    // TODO: wire geometry_core search/layout execution here.
    println!("Running backend-only mode (no viewer).");
}

#[cfg(feature = "viewer")]
fn load_viewer_config() -> viewer::ViewerConfig {
    use std::fs;
    use std::path::PathBuf;

    let path = std::env::var("ASSET_IMPORT_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("app/config/asset_import.toml"));

    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) => {
        eprintln!(
            "Failed to read asset import config at {}: {}",
            path.display(),
            err
        );
            return viewer::ViewerConfig::default();
        }
    };

    let mut config: viewer::ViewerConfig = toml::from_str(&data).unwrap_or_else(|err| {
        eprintln!(
            "Failed to parse asset import config at {}: {}",
            path.display(),
            err
        );
        viewer::ViewerConfig::default()
    });

    resolve_viewer_paths(&path, &mut config);
    config
}

#[cfg(feature = "viewer")]
fn resolve_viewer_paths(config_path: &std::path::Path, config: &mut viewer::ViewerConfig) {
    let base = config_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    for placement in &mut config.assets {
        let p = std::path::Path::new(&placement.obj_path);
        if p.is_absolute() {
            continue;
        }
        let joined = base.join(p);
        placement.obj_path = joined.to_string_lossy().into_owned();
    }
}
