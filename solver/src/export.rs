use crate::config::SceneConfig;

pub fn export_scene_json(config: &SceneConfig) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("scene.json");
    let text = serde_json::to_string_pretty(config)
        .map_err(|err| format!("Failed to serialize scene json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}

pub fn export_transforms_json() -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("transforms.json");
    if out_path.exists() {
        return Ok(());
    }
    let payload = serde_json::json!({
        "version": 1,
        "space_meshes": [],
        "placements": []
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize transforms json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}

pub fn export_debug_points_json(points: &[[f32; 3]]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_points.json");
    let payload = serde_json::json!({
        "points": points,
        "color": [0.9, 0.8, 0.2],
        "radius": 6.0
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug points json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}

pub fn export_debug_boundary_json(points: &[[f32; 3]]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_boundary.json");
    let payload = serde_json::json!({
        "points": points,
        "color": [0.8, 0.2, 0.15]
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug boundary json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}
