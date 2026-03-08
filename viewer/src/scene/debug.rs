use bevy::log::{error, info};
use bevy::prelude::*;

use super::{DebugBoundaryPoints, DebugEntities, DebugSpaceBoundaries, DebugNfpData, DebugIfpData};

#[derive(serde::Deserialize)]
struct DebugPointsFile {
    points: Vec<[f32; 3]>,
    #[serde(default)]
    color: Option<[f32; 3]>,
    #[serde(default)]
    radius: Option<f32>,
}

pub(super) fn load_debug_points_from_path<T: DebugEntities>(
    path: &str,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entities: &mut T,
) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Debug points not loaded ({}): {}", path, err);
            return;
        }
    };
    let parsed: DebugPointsFile = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Failed to parse debug points {}: {}", path, err);
            return;
        }
    };

    for e in entities.entities_mut().drain(..) {
        commands.entity(e).despawn_recursive();
    }

    info!(
        "Loaded debug points: {} points, first={:?}",
        parsed.points.len(),
        parsed.points.first().copied()
    );
    if let Some((min, max)) = bounds_for_points(&parsed.points) {
        info!("Debug points bounds: min={:?} max={:?}", min, max);
    }

    let color = parsed.color.unwrap_or([0.9, 0.8, 0.2]);
    let radius = parsed.radius.unwrap_or(6.0);
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(color[0], color[1], color[2]),
        unlit: true,
        ..default()
    });
    let sphere = meshes.add(Mesh::from(Sphere::new(1.0)));

    for p in parsed.points {
        let id = commands
            .spawn(PbrBundle {
                mesh: sphere.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(p[0], p[1], p[2]))
                    .with_scale(Vec3::splat(radius)),
                ..default()
            })
            .id();
        entities.entities_mut().push(id);
    }
}

#[derive(serde::Deserialize)]
struct DebugBoundaryFile {
    points: Vec<[f32; 3]>,
    #[serde(default)]
    color: Option<[f32; 3]>,
}

pub(super) fn load_debug_boundary_from_path(path: &str, boundary: &mut DebugBoundaryPoints) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Debug boundary not loaded ({}): {}", path, err);
            return;
        }
    };
    let parsed: DebugBoundaryFile = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Failed to parse debug boundary {}: {}", path, err);
            return;
        }
    };

    if parsed.points.len() < 2 {
        info!("Debug boundary has <2 points ({}).", path);
        boundary.points.clear();
        return;
    }
    info!(
        "Loaded debug boundary: {} points, first={:?}",
        parsed.points.len(),
        parsed.points.first().copied()
    );
    if let Some((min, max)) = bounds_for_points(&parsed.points) {
        info!("Debug boundary bounds: min={:?} max={:?}", min, max);
    }

    let color = parsed.color.unwrap_or([0.8, 0.2, 0.15]);
    boundary.color = Color::rgb(color[0], color[1], color[2]);
    boundary.y_offset = 5.0;
    boundary.points = parsed
        .points
        .into_iter()
        .map(|p| Vec3::new(p[0], p[1], p[2]))
        .collect();
}

#[derive(serde::Deserialize)]
struct DebugSpaceBoundaryFile {
    loops: Vec<Vec<[f32; 3]>>,
    #[serde(default)]
    color: Option<[f32; 3]>,
}

pub(super) fn load_space_boundaries_from_path(
    path: &str,
    boundaries: &mut DebugSpaceBoundaries,
) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Debug space boundary not loaded ({}): {}", path, err);
            return;
        }
    };
    let parsed: DebugSpaceBoundaryFile = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Failed to parse debug space boundary {}: {}", path, err);
            return;
        }
    };

    let color = parsed.color.unwrap_or([0.2, 0.9, 0.9]);
    boundaries.color = Color::rgb(color[0], color[1], color[2]);
    boundaries.y_offset = 2.0;
    boundaries.loops = parsed
        .loops
        .into_iter()
        .map(|loop_pts| loop_pts.into_iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect())
        .collect();
}

fn bounds_for_points(points: &[[f32; 3]]) -> Option<(Vec3, Vec3)> {
    if points.is_empty() {
        return None;
    }
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in points {
        min.x = min.x.min(p[0]);
        min.y = min.y.min(p[1]);
        min.z = min.z.min(p[2]);
        max.x = max.x.max(p[0]);
        max.y = max.y.max(p[1]);
        max.z = max.z.max(p[2]);
    }
    Some((min, max))
}

#[derive(serde::Deserialize)]
struct DebugNfpFile {
    #[serde(default)]
    rect: Vec<[f32; 3]>,
    #[serde(default)]
    nfp: Vec<[f32; 3]>,
    #[serde(default)]
    rect_loops: Vec<Vec<[f32; 3]>>,
    #[serde(default)]
    nfp_loops: Vec<Vec<[f32; 3]>>,
}

pub(super) fn load_nfp_from_path(path: &str, out: &mut DebugNfpData) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Debug NFP not loaded ({}): {}", path, err);
            return;
        }
    };
    let parsed: DebugNfpFile = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Failed to parse debug NFP {}: {}", path, err);
            return;
        }
    };
    let rect_loops = if !parsed.rect_loops.is_empty() {
        parsed.rect_loops
    } else if !parsed.rect.is_empty() {
        vec![parsed.rect]
    } else {
        Vec::new()
    };
    let nfp_loops = if !parsed.nfp_loops.is_empty() {
        parsed.nfp_loops
    } else if !parsed.nfp.is_empty() {
        vec![parsed.nfp]
    } else {
        Vec::new()
    };

    out.rect_loops = rect_loops
        .into_iter()
        .map(|loop_pts| loop_pts.into_iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect())
        .collect();
    out.nfp_loops = nfp_loops
        .into_iter()
        .map(|loop_pts| loop_pts.into_iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect())
        .collect();
}

#[derive(serde::Deserialize)]
struct DebugIfpFile {
    #[serde(default)]
    ifp_loops: Vec<Vec<[f32; 3]>>,
    #[serde(default)]
    instance_loops: Vec<Vec<[f32; 3]>>,
}

pub(super) fn load_ifp_from_path(path: &str, out: &mut DebugIfpData) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Debug IFP not loaded ({}): {}", path, err);
            return;
        }
    };
    let parsed: DebugIfpFile = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Failed to parse debug IFP {}: {}", path, err);
            return;
        }
    };
    out.loops = parsed
        .ifp_loops
        .into_iter()
        .map(|loop_pts| loop_pts.into_iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect())
        .collect();
    out.instance_loops = parsed
        .instance_loops
        .into_iter()
        .map(|loop_pts| loop_pts.into_iter().map(|p| Vec3::new(p[0], p[1], p[2])).collect())
        .collect();
}
