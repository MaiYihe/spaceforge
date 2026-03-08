use crate::config::SceneConfig;
use geometry_core::models::placement_region_instance::PlacementTransform;

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

pub fn export_transforms_json(placements: &[PlacementTransform]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("transforms.json");
    let payload = serde_json::json!({
        "version": 1,
        "space_meshes": [],
        "placements": placements.iter().enumerate().map(|(index, t)| {
            serde_json::json!({ "index": index, "matrix": t.matrix })
        }).collect::<Vec<_>>()
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

pub fn export_debug_forbidden_points_json(points: &[[f32; 3]]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_forbidden_points.json");
    let payload = serde_json::json!({
        "points": points,
        "color": [0.95, 0.25, 0.2],
        "radius": 6.0
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug forbidden points json: {err}"))?;
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

pub fn export_space_boundaries_json(loops: &[Vec<[f32; 3]>]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_space_boundary.json");
    let payload = serde_json::json!({
        "loops": loops,
        "color": [0.2, 0.9, 0.9]
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug space boundary json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}

pub fn export_nfp_debug_json(rect: &[[f32; 3]], nfp: &[Vec<[f32; 3]>]) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_nfp.json");
    let payload = serde_json::json!({
        "rect_loops": [rect],
        "nfp_loops": nfp
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug nfp json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}

pub fn export_ifp_debug_json(
    ifp: &[Vec<[f32; 3]>],
    sample_points: &[[f32; 3]],
    instance_loops: &[Vec<[f32; 3]>],
) -> Result<(), String> {
    let out_dir = std::path::Path::new("/tmp/spaceforge");
    std::fs::create_dir_all(out_dir)
        .map_err(|err| format!("Failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("debug_ifp.json");
    let payload = serde_json::json!({
        "ifp_loops": ifp,
        "sample_points": sample_points,
        "instance_loops": instance_loops
    });
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Failed to serialize debug ifp json: {err}"))?;
    std::fs::write(&out_path, text)
        .map_err(|err| format!("Failed to write {}: {err}", out_path.display()))?;
    Ok(())
}
