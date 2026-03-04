#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SceneConfig {
    pub space_usda_path: String,
    pub placement_region_usda_dir: String,
    pub regions_type_path: String,
    pub usda_scale: f32,
}

pub fn load_scene_config(path: &str) -> Result<SceneConfig, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read config at {path}: {err}"))?;
    let mut config: SceneConfig =
        toml::from_str(&data).map_err(|err| format!("Failed to parse config: {err}"))?;

    let base = std::path::Path::new(path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    config.space_usda_path = resolve_path(base, &config.space_usda_path);
    config.placement_region_usda_dir = resolve_path(base, &config.placement_region_usda_dir);
    config.regions_type_path = resolve_path(base, &config.regions_type_path);
    config.space_usda_path = canonicalize_if_possible(&config.space_usda_path);
    config.placement_region_usda_dir = canonicalize_if_possible(&config.placement_region_usda_dir);
    config.regions_type_path = canonicalize_if_possible(&config.regions_type_path);
    Ok(config)
}

fn resolve_path(base: &std::path::Path, raw: &str) -> String {
    let p = std::path::Path::new(raw);
    if p.is_absolute() {
        return raw.to_string();
    }
    base.join(p).to_string_lossy().into_owned()
}

fn canonicalize_if_possible(path: &str) -> String {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}
