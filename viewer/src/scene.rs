use bevy::log::{error, info};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use crossbeam_channel::{Receiver, Sender};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

use crate::camera::OrbitCamera;
use crate::config::{SceneFileConfig, ViewerConfig};
use assets_import::{
    load_placement_regions_from_dir, load_regions_type_registry, load_space_model_from_usda,
};
use geometry_core::models::placement_region::PlacementRegion;
use geometry_core::models::space::Space;

#[derive(Resource)]
pub struct SceneInfo {
    pub center: Vec3,
    pub focus_distance: f32,
}

#[derive(Clone)]
pub struct ScenePayload {
    pub space: Space,
    pub placements: Vec<PlacementRegion>,
}

#[derive(Resource)]
pub struct SceneReceiver(pub Option<Receiver<ScenePayload>>);

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) enum PlacementRenderMode {
    Mesh,
    Voxels,
}

#[derive(Resource)]
pub(crate) struct FileWatchResource {
    rx: Receiver<notify::Result<notify::Event>>,
    _watcher: RecommendedWatcher,
    scene_path: String,
    transforms_path: String,
    debug_points_path: String,
    debug_boundary_path: String,
}

#[derive(Resource, Default)]
pub(crate) struct SceneEntities {
    entities: Vec<Entity>,
}

#[derive(Resource, Default)]
pub(crate) struct DebugPointsEntities {
    entities: Vec<Entity>,
}

#[derive(Resource, Default)]
pub(crate) struct DebugBoundaryPoints {
    pub points: Vec<Vec3>,
    pub color: Color,
    pub y_offset: f32,
}
#[derive(Default, Resource)]
pub(crate) struct SceneTransforms {
    space_meshes: Vec<IndexedTransform>,
    placements: Vec<IndexedTransform>,
}

#[derive(Clone)]
struct IndexedTransform {
    index: usize,
    matrix: [[f32; 4]; 4],
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _config: Res<ViewerConfig>,
) {
    commands.insert_resource(SceneEntities::default());
    commands.insert_resource(DebugPointsEntities::default());
    commands.insert_resource(DebugBoundaryPoints::default());
    commands.insert_resource(load_render_mode());
    commands.insert_resource(load_transforms_resource());

    let center = Vec3::ZERO;
    let focus_distance = 1000.0;

    // Light + Camera
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(2000.0, 2000.0, 2000.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(center + Vec3::new(0.0, 0.0, focus_distance))
            .looking_at(center, Vec3::Y),
        ..default()
    });

    commands.insert_resource(OrbitCamera {
        yaw: 0.0,
        pitch: 0.2,
        distance: focus_distance,
        target: center,
    });
    commands.insert_resource(SceneInfo {
        center,
        focus_distance,
    });

    let _ = (&mut meshes, &mut materials, &mut commands);
}

pub fn init_file_watcher(mut commands: Commands) {
    let scene_path = std::env::var("SCENE_JSON").unwrap_or_else(|_| "/tmp/spaceforge/scene.json".into());
    let transforms_path =
        std::env::var("SCENE_TRANSFORMS").unwrap_or_else(|_| "/tmp/spaceforge/transforms.json".into());
    let debug_points_path =
        std::env::var("SCENE_DEBUG_POINTS").unwrap_or_else(|_| "/tmp/spaceforge/debug_points.json".into());
    let debug_boundary_path =
        std::env::var("SCENE_DEBUG_BOUNDARY").unwrap_or_else(|_| "/tmp/spaceforge/debug_boundary.json".into());
    match create_file_watcher(
        &scene_path,
        &transforms_path,
        &debug_points_path,
        &debug_boundary_path,
    ) {
        Ok(resource) => commands.insert_resource(resource),
        Err(err) => error!("Failed to init file watcher: {}", err),
    }
}

pub fn load_scene_from_file(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cam: ResMut<OrbitCamera>,
    mut scene_info: ResMut<SceneInfo>,
    mut entities: ResMut<SceneEntities>,
    mut debug_entities: ResMut<DebugPointsEntities>,
    mut boundary_points: ResMut<DebugBoundaryPoints>,
    transforms: Res<SceneTransforms>,
    render_mode: Res<PlacementRenderMode>,
) {
    let path = std::env::var("SCENE_JSON").unwrap_or_else(|_| "/tmp/spaceforge/scene.json".into());
    if let Some(payload) = load_scene_from_json(&path) {
        apply_payload(
            &payload,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut cam,
            &mut scene_info,
            &mut entities,
            &transforms,
            *render_mode,
        );
    }

    let debug_path =
        std::env::var("SCENE_DEBUG_POINTS").unwrap_or_else(|_| "/tmp/spaceforge/debug_points.json".into());
    load_debug_points_from_path(
        &debug_path,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut debug_entities,
    );

    let boundary_path = std::env::var("SCENE_DEBUG_BOUNDARY")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_boundary.json".into());
    load_debug_boundary_from_path(
        &boundary_path,
        &mut boundary_points,
    );
}

pub fn apply_scene_updates(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cam: ResMut<OrbitCamera>,
    mut scene_info: ResMut<SceneInfo>,
    mut entities: ResMut<SceneEntities>,
    transforms: Res<SceneTransforms>,
    receiver: Res<SceneReceiver>,
    render_mode: Res<PlacementRenderMode>,
) {
    let Some(rx) = receiver.0.as_ref() else {
        return;
    };

    let mut latest = None;
    while let Ok(payload) = rx.try_recv() {
        latest = Some(payload);
    }

    let Some(payload) = latest else {
        return;
    };

        apply_payload(
            &payload,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut cam,
            &mut scene_info,
            &mut entities,
            &transforms,
            *render_mode,
        );
}

pub fn apply_file_watch_updates(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cam: ResMut<OrbitCamera>,
    mut scene_info: ResMut<SceneInfo>,
    mut entities: ResMut<SceneEntities>,
    mut debug_entities: ResMut<DebugPointsEntities>,
    mut boundary_points: ResMut<DebugBoundaryPoints>,
    mut transforms: ResMut<SceneTransforms>,
    watcher: Res<FileWatchResource>,
    render_mode: Res<PlacementRenderMode>,
) {
    let mut changed = false;
    let mut debug_changed = false;
    let mut boundary_changed = false;
    while let Ok(event) = watcher.rx.try_recv() {
        match event {
            Ok(event) => {
                for path in event.paths.iter() {
                    let path = path.to_string_lossy();
                    if path == watcher.scene_path || path == watcher.transforms_path {
                        changed = true;
                    }
                    if path == watcher.debug_points_path {
                        debug_changed = true;
                        break;
                    }
                    if path == watcher.debug_boundary_path {
                        boundary_changed = true;
                        break;
                    }
                }
            }
            Err(err) => {
                error!("File watch error: {}", err);
            }
        }
    }

    if changed {
        *transforms = load_transforms_from_path(&watcher.transforms_path);
        if let Some(payload) = load_scene_from_json(&watcher.scene_path) {
            apply_payload(
                &payload,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut cam,
                &mut scene_info,
                &mut entities,
                &transforms,
                *render_mode,
            );
        }
    }

    if debug_changed {
        load_debug_points_from_path(
            &watcher.debug_points_path,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut debug_entities,
        );
    }

    if boundary_changed {
        load_debug_boundary_from_path(
            &watcher.debug_boundary_path,
            &mut boundary_points,
        );
    }
}

fn apply_payload(
    payload: &ScenePayload,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cam: &mut OrbitCamera,
    scene_info: &mut SceneInfo,
    entities: &mut SceneEntities,
    transforms: &SceneTransforms,
    render_mode: PlacementRenderMode,
) {
    for e in entities.entities.drain(..) {
        commands.entity(e).despawn_recursive();
    }

    let (min, max) = compute_bounds(&payload.space);
    let center = (min + max) * 0.5;
    let extent = max - min;
    let size = vec3(extent.x.abs(), extent.y.abs(), extent.z.abs());
    let max_extent = size.x.max(size.y).max(size.z);
    let focus_distance = (max_extent * 1.5).max(100.0);

    scene_info.center = center;
    scene_info.focus_distance = focus_distance;
    cam.target = center;
    cam.distance = focus_distance;

    for (idx, (mesh, meta)) in payload
        .space
        .meshes
        .iter()
        .zip(payload.space.surface_metas.iter())
        .enumerate()
    {
        let mut positions = mesh.positions.clone();
        if let Some(xform) = find_transform(&transforms.space_meshes, idx) {
            apply_transform_positions(&mut positions, xform);
        }
        let mesh = mesh_from_geometry(positions, mesh.indices.clone());
        let mask_bits = meta.regions_type_mask.bits();
        let color = color_from_mask(mask_bits);
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.7,
            metallic: 0.0,
            ..default()
        });
        let id = commands
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material,
                ..default()
            })
            .id();
        entities.entities.push(id);
    }

    for (idx, placement) in payload.placements.iter().enumerate() {
        let placement_transform = find_transform(&transforms.placements, idx);
        let restricted_mesh = mesh_from_geometry(
            apply_optional_transform(
                placement.regions.restricted_region.mesh.positions.clone(),
                placement_transform,
            ),
            placement.regions.restricted_region.mesh.indices.clone(),
        );
        let restricted_material = materials.add(StandardMaterial {
            base_color: Color::rgba(0.2, 0.6, 1.0, 0.4),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let id = commands
            .spawn(PbrBundle {
                mesh: meshes.add(restricted_mesh),
                material: restricted_material,
                ..default()
            })
            .id();
        entities.entities.push(id);

        match render_mode {
            PlacementRenderMode::Mesh => {
                let forbidden_mesh = mesh_from_geometry(
                    apply_optional_transform(
                        placement.regions.forbidden_region.mesh.positions.clone(),
                        placement_transform,
                    ),
                    placement.regions.forbidden_region.mesh.indices.clone(),
                );
                let forbidden_material = materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 0.3, 0.2, 0.4),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });
                let id = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(forbidden_mesh),
                        material: forbidden_material,
                        ..default()
                    })
                    .id();
                entities.entities.push(id);
            }
            PlacementRenderMode::Voxels => {
                if let Some(sdf) = &placement.regions.forbidden_region.sdf {
                    spawn_voxels(
                        commands,
                        meshes,
                        materials,
                        entities,
                        sdf,
                        placement_transform,
                        Color::rgba(1.0, 0.3, 0.2, 0.6),
                    );
                }
            }
        }

        if !placement.visual.footprint_2d.positions.is_empty()
            && !placement.visual.footprint_2d.indices.is_empty()
        {
            let footprint_mesh = mesh_from_geometry(
                apply_optional_transform(
                    placement.visual.footprint_2d.positions.clone(),
                    placement_transform,
                ),
                placement.visual.footprint_2d.indices.clone(),
            );
            let footprint_material = materials.add(StandardMaterial {
                base_color: Color::rgba(0.1, 0.9, 0.5, 0.4),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            let id = commands
                .spawn(PbrBundle {
                    mesh: meshes.add(footprint_mesh),
                    material: footprint_material,
                    ..default()
                })
                .id();
            entities.entities.push(id);
        }
    }
}

fn load_render_mode() -> PlacementRenderMode {
    let mode = std::env::var("PLACEMENT_REGION_RENDER").unwrap_or_else(|_| "mesh".to_string());
    match mode.to_ascii_lowercase().as_str() {
        "voxels" | "voxel" | "sdf" => PlacementRenderMode::Voxels,
        _ => PlacementRenderMode::Mesh,
    }
}

fn spawn_voxels(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entities: &mut SceneEntities,
    sdf: &geometry_core::models::placement_region::SdfGrid,
    transform: Option<&[[f32; 4]; 4]>,
    color: Color,
) {
    let centers = match sdf.grid.active_voxel_centers() {
        Ok(c) => c,
        Err(_) => return,
    };
    if centers.is_empty() {
        log::info!("viewer::scene: voxel centers empty");
        return;
    }
    log::info!("viewer::scene: spawning voxels count={}", centers.len());
    let voxel_size = sdf.voxel_size.max(1.0);
    let mesh_handle = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    });
    for p in centers {
        let mut pos = [p[0], p[1], p[2]];
        if let Some(m) = transform {
            apply_transform_point(&mut pos, m);
        }
        let id = commands
            .spawn(PbrBundle {
                mesh: mesh_handle.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(pos[0], pos[1], pos[2]))
                    .with_scale(Vec3::splat(voxel_size)),
                ..default()
            })
            .id();
        entities.entities.push(id);
    }
}

fn mesh_from_geometry(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    let normals = compute_vertex_normals(&positions, &indices);
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32, 0.0, 0.0]; positions.len()];

    let mut i = 0;
    while i + 2 < indices.len() {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;
        i += 3;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let p0 = Vec3::from(positions[i0]);
        let p1 = Vec3::from(positions[i1]);
        let p2 = Vec3::from(positions[i2]);

        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let n = e1.cross(e2);

        normals[i0][0] += n.x;
        normals[i0][1] += n.y;
        normals[i0][2] += n.z;
        normals[i1][0] += n.x;
        normals[i1][1] += n.y;
        normals[i1][2] += n.z;
        normals[i2][0] += n.x;
        normals[i2][1] += n.y;
        normals[i2][2] += n.z;
    }

    for n in &mut normals {
        let v = Vec3::from(*n).normalize_or_zero();
        *n = [v.x, v.y, v.z];
    }

    normals
}

fn color_from_mask(mask: u32) -> Color {
    if mask == 0 {
        return Color::rgb(0.85, 0.85, 0.85);
    }

    let mut x = mask ^ 0x9E37_79B9;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    let r = ((x & 0xFF) as f32) / 255.0;
    let g = (((x >> 8) & 0xFF) as f32) / 255.0;
    let b = (((x >> 16) & 0xFF) as f32) / 255.0;

    let r = 0.2 + r * 0.7;
    let g = 0.2 + g * 0.7;
    let b = 0.2 + b * 0.7;
    Color::rgb(r, g, b)
}

fn compute_bounds(space: &Space) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for mesh in &space.meshes {
        for p in &mesh.positions {
            min.x = min.x.min(p[0]);
            min.y = min.y.min(p[1]);
            min.z = min.z.min(p[2]);
            max.x = max.x.max(p[0]);
            max.y = max.y.max(p[1]);
            max.z = max.z.max(p[2]);
        }
    }

    if !min.x.is_finite() || !max.x.is_finite() {
        (Vec3::ZERO, Vec3::ZERO)
    } else {
        (min, max)
    }
}

fn load_scene_from_json(path: &str) -> Option<ScenePayload> {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to read scene config {}: {}", path, err);
            return None;
        }
    };
    let mut config: SceneFileConfig = match serde_json::from_str(&data) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to parse scene config {}: {}", path, err);
            return None;
        }
    };

    let base = std::path::Path::new(path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    config.space_usda_path = resolve_path(base, &config.space_usda_path);
    config.placement_region_usda_dir = resolve_path(base, &config.placement_region_usda_dir);
    config.regions_type_path = resolve_path(base, &config.regions_type_path);

    info!(
        "Scene config resolved: space={} placement_dir={} regions_type={} scale={}",
        config.space_usda_path,
        config.placement_region_usda_dir,
        config.regions_type_path,
        config.usda_scale
    );

    let regions_type_ids = match load_regions_type_registry(&config.regions_type_path) {
        Ok(registry) => registry,
        Err(err) => {
            error!("Failed to load regions type registry: {}", err);
            return None;
        }
    };
    let space = match load_space_model_from_usda(
        &config.space_usda_path,
        &regions_type_ids,
        config.usda_scale,
    ) {
        Ok(space) => space,
        Err(err) => {
            error!("Failed to load Space from {}: {}", config.space_usda_path, err);
            return None;
        }
    };
    let placements = match load_placement_regions_from_dir(
        std::path::Path::new(&config.placement_region_usda_dir),
        &regions_type_ids,
        config.usda_scale,
    ) {
        Ok(placements) => placements,
        Err(err) => {
            error!(
                "Failed to load PlacementRegions from {}: {}",
                config.placement_region_usda_dir, err
            );
            return None;
        }
    };

    Some(ScenePayload { space, placements })
}

fn resolve_path(base: &std::path::Path, raw: &str) -> String {
    let p = std::path::Path::new(raw);
    if p.is_absolute() {
        return raw.to_string();
    }
    base.join(p).to_string_lossy().into_owned()
}

fn load_transforms_resource() -> SceneTransforms {
    let path = std::env::var("SCENE_TRANSFORMS")
        .unwrap_or_else(|_| "/tmp/spaceforge/transforms.json".into());
    load_transforms_from_path(&path)
}

fn load_transforms_from_path(path: &str) -> SceneTransforms {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => return SceneTransforms::default(),
    };

    #[derive(serde::Deserialize)]
    struct TransformFile {
        #[allow(dead_code)]
        version: u32,
        space_meshes: Vec<TransformEntry>,
        placements: Vec<TransformEntry>,
    }

    #[derive(serde::Deserialize)]
    struct TransformEntry {
        index: usize,
        matrix: [[f32; 4]; 4],
    }

    let parsed: TransformFile = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(_) => return SceneTransforms::default(),
    };

    SceneTransforms {
        space_meshes: parsed
            .space_meshes
            .into_iter()
            .map(|e| IndexedTransform {
                index: e.index,
                matrix: e.matrix,
            })
            .collect(),
        placements: parsed
            .placements
            .into_iter()
            .map(|e| IndexedTransform {
                index: e.index,
                matrix: e.matrix,
            })
            .collect(),
    }
}

fn create_file_watcher(
    scene_path: &str,
    transforms_path: &str,
    debug_points_path: &str,
    debug_boundary_path: &str,
) -> Result<FileWatchResource, String> {
    let (tx, rx): (Sender<notify::Result<notify::Event>>, Receiver<notify::Result<notify::Event>>) =
        crossbeam_channel::unbounded();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        NotifyConfig::default(),
    )
    .map_err(|err| format!("create watcher failed: {err}"))?;
    watcher
        .watch(std::path::Path::new(scene_path), RecursiveMode::NonRecursive)
        .map_err(|err| format!("watch scene.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(transforms_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch transforms.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_points_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_points.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_boundary_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_boundary.json failed: {err}"))?;

    Ok(FileWatchResource {
        rx,
        _watcher: watcher,
        scene_path: scene_path.to_string(),
        transforms_path: transforms_path.to_string(),
        debug_points_path: debug_points_path.to_string(),
        debug_boundary_path: debug_boundary_path.to_string(),
    })
}

#[derive(serde::Deserialize)]
struct DebugPointsFile {
    points: Vec<[f32; 3]>,
    #[serde(default)]
    color: Option<[f32; 3]>,
    #[serde(default)]
    radius: Option<f32>,
}

fn load_debug_points_from_path(
    path: &str,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    entities: &mut DebugPointsEntities,
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

    for e in entities.entities.drain(..) {
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
        entities.entities.push(id);
    }
}

#[derive(serde::Deserialize)]
struct DebugBoundaryFile {
    points: Vec<[f32; 3]>,
    #[serde(default)]
    color: Option<[f32; 3]>,
}

fn load_debug_boundary_from_path(
    path: &str,
    boundary: &mut DebugBoundaryPoints,
) {
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
        info!(
            "Debug boundary bounds: min={:?} max={:?}",
            min, max
        );
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

fn find_transform(transforms: &[IndexedTransform], index: usize) -> Option<&[[f32; 4]; 4]> {
    transforms
        .iter()
        .find(|t| t.index == index)
        .map(|t| &t.matrix)
}

fn apply_optional_transform(
    mut positions: Vec<[f32; 3]>,
    transform: Option<&[[f32; 4]; 4]>,
) -> Vec<[f32; 3]> {
    if let Some(m) = transform {
        apply_transform_positions(&mut positions, m);
    }
    positions
}

fn apply_transform_positions(positions: &mut [[f32; 3]], m: &[[f32; 4]; 4]) {
    for p in positions.iter_mut() {
        apply_transform_point(p, m);
    }
}

fn apply_transform_point(p: &mut [f32; 3], m: &[[f32; 4]; 4]) {
    let x = p[0];
    let y = p[1];
    let z = p[2];
    p[0] = x * m[0][0] + y * m[1][0] + z * m[2][0] + m[3][0];
    p[1] = x * m[0][1] + y * m[1][1] + z * m[2][1] + m[3][1];
    p[2] = x * m[0][2] + y * m[1][2] + z * m[2][2] + m[3][2];
}
