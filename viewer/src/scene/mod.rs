use bevy::log::error;
use bevy::prelude::*;
use crossbeam_channel::Receiver;

use crate::camera::OrbitCamera;
use crate::config::ViewerConfig;
use geometry_core::models::placement_region::PlacementRegion;
use geometry_core::models::space::Space;

mod debug;
mod io;
mod render;
mod transform;
mod watch;

#[derive(Resource)]
pub struct SceneInfo {
    pub center: Vec3,
    pub focus_distance: f32,
    pub camera_initialized: bool,
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
    _watcher: notify::RecommendedWatcher,
    scene_path: String,
    transforms_path: String,
    debug_points_path: String,
    debug_forbidden_points_path: String,
    debug_space_boundary_path: String,
    debug_nfp_path: String,
    debug_ifp_path: String,
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
pub(crate) struct DebugForbiddenPointsEntities {
    entities: Vec<Entity>,
}

#[derive(Resource, Default)]
pub(crate) struct DebugBoundaryPoints {
    pub points: Vec<Vec3>,
    pub color: Color,
    pub y_offset: f32,
}

#[derive(Resource, Default)]
pub(crate) struct DebugSpaceBoundaries {
    pub loops: Vec<Vec<Vec3>>,
    pub color: Color,
    pub y_offset: f32,
}

#[derive(Resource, Default)]
pub(crate) struct DebugNfpData {
    pub rect_loops: Vec<Vec<Vec3>>,
    pub nfp_loops: Vec<Vec<Vec3>>,
}

#[derive(Resource, Default)]
pub(crate) struct DebugIfpData {
    pub loops: Vec<Vec<Vec3>>,
    pub instance_loops: Vec<Vec<Vec3>>,
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

trait DebugEntities {
    fn entities_mut(&mut self) -> &mut Vec<Entity>;
}

impl DebugEntities for DebugPointsEntities {
    fn entities_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}

impl DebugEntities for DebugForbiddenPointsEntities {
    fn entities_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}

impl<'a, T: DebugEntities + Resource> DebugEntities for ResMut<'a, T> {
    fn entities_mut(&mut self) -> &mut Vec<Entity> {
        self.bypass_change_detection().entities_mut()
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _config: Res<ViewerConfig>,
) {
    commands.insert_resource(SceneEntities::default());
    commands.insert_resource(DebugPointsEntities::default());
    commands.insert_resource(DebugForbiddenPointsEntities::default());
    commands.insert_resource(DebugBoundaryPoints::default());
    commands.insert_resource(DebugSpaceBoundaries::default());
    commands.insert_resource(DebugNfpData::default());
    commands.insert_resource(DebugIfpData::default());
    commands.insert_resource(render::load_render_mode());
    commands.insert_resource(io::load_transforms_resource());

    let center = Vec3::ZERO;
    let focus_distance = 1000.0;

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
        camera_initialized: false,
    });

    let _ = (&mut meshes, &mut materials, &mut commands);
}

pub fn init_file_watcher(mut commands: Commands) {
    let scene_path = std::env::var("SCENE_JSON").unwrap_or_else(|_| "/tmp/spaceforge/scene.json".into());
    let transforms_path =
        std::env::var("SCENE_TRANSFORMS").unwrap_or_else(|_| "/tmp/spaceforge/transforms.json".into());
    let debug_points_path =
        std::env::var("SCENE_DEBUG_POINTS").unwrap_or_else(|_| "/tmp/spaceforge/debug_points.json".into());
    let debug_forbidden_points_path = std::env::var("SCENE_FORBIDDEN_POINTS")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_forbidden_points.json".into());
    let debug_space_boundary_path = std::env::var("SCENE_SPACE_BOUNDARY")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_space_boundary.json".into());
    let debug_nfp_path =
        std::env::var("SCENE_DEBUG_NFP").unwrap_or_else(|_| "/tmp/spaceforge/debug_nfp.json".into());
    let debug_ifp_path =
        std::env::var("SCENE_DEBUG_IFP").unwrap_or_else(|_| "/tmp/spaceforge/debug_ifp.json".into());
    let debug_boundary_path =
        std::env::var("SCENE_DEBUG_BOUNDARY").unwrap_or_else(|_| "/tmp/spaceforge/debug_boundary.json".into());
    match watch::create_file_watcher(
        &scene_path,
        &transforms_path,
        &debug_points_path,
        &debug_forbidden_points_path,
        &debug_space_boundary_path,
        &debug_nfp_path,
        &debug_ifp_path,
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
    mut debug_forbidden_entities: ResMut<DebugForbiddenPointsEntities>,
    mut boundary_points: ResMut<DebugBoundaryPoints>,
    mut space_boundaries: ResMut<DebugSpaceBoundaries>,
    mut nfp_data: ResMut<DebugNfpData>,
    mut ifp_data: ResMut<DebugIfpData>,
    transforms: Res<SceneTransforms>,
    render_mode: Res<PlacementRenderMode>,
) {
    let path = std::env::var("SCENE_JSON").unwrap_or_else(|_| "/tmp/spaceforge/scene.json".into());
    if let Some(payload) = io::load_scene_from_json(&path) {
        render::apply_payload(
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
    debug::load_debug_points_from_path(
        &debug_path,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut debug_entities,
    );

    let forbidden_debug_path = std::env::var("SCENE_FORBIDDEN_POINTS")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_forbidden_points.json".into());
    debug::load_debug_points_from_path(
        &forbidden_debug_path,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut debug_forbidden_entities,
    );

    let space_boundary_path = std::env::var("SCENE_SPACE_BOUNDARY")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_space_boundary.json".into());
    debug::load_space_boundaries_from_path(&space_boundary_path, &mut space_boundaries);

    let nfp_path =
        std::env::var("SCENE_DEBUG_NFP").unwrap_or_else(|_| "/tmp/spaceforge/debug_nfp.json".into());
    debug::load_nfp_from_path(&nfp_path, &mut nfp_data);

    let ifp_path =
        std::env::var("SCENE_DEBUG_IFP").unwrap_or_else(|_| "/tmp/spaceforge/debug_ifp.json".into());
    debug::load_ifp_from_path(&ifp_path, &mut ifp_data);

    let boundary_path = std::env::var("SCENE_DEBUG_BOUNDARY")
        .unwrap_or_else(|_| "/tmp/spaceforge/debug_boundary.json".into());
    debug::load_debug_boundary_from_path(&boundary_path, &mut boundary_points);
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

    render::apply_payload(
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
    mut debug_forbidden_entities: ResMut<DebugForbiddenPointsEntities>,
    mut boundary_points: ResMut<DebugBoundaryPoints>,
    mut space_boundaries: ResMut<DebugSpaceBoundaries>,
    mut nfp_data: ResMut<DebugNfpData>,
    mut ifp_data: ResMut<DebugIfpData>,
    mut transforms: ResMut<SceneTransforms>,
    watcher: Res<FileWatchResource>,
    render_mode: Res<PlacementRenderMode>,
) {
    let mut changed = false;
    let mut debug_changed = false;
    let mut forbidden_debug_changed = false;
    let mut space_boundary_changed = false;
    let mut nfp_changed = false;
    let mut ifp_changed = false;
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
                    if path == watcher.debug_forbidden_points_path {
                        forbidden_debug_changed = true;
                        break;
                    }
                    if path == watcher.debug_space_boundary_path {
                        space_boundary_changed = true;
                        break;
                    }
                    if path == watcher.debug_nfp_path {
                        nfp_changed = true;
                        break;
                    }
                    if path == watcher.debug_ifp_path {
                        ifp_changed = true;
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
        *transforms = io::load_transforms_from_path(&watcher.transforms_path);
        if let Some(payload) = io::load_scene_from_json(&watcher.scene_path) {
            render::apply_payload(
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
        debug::load_debug_points_from_path(
            &watcher.debug_points_path,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut debug_entities,
        );
    }

    if forbidden_debug_changed {
        debug::load_debug_points_from_path(
            &watcher.debug_forbidden_points_path,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut debug_forbidden_entities,
        );
    }
    if space_boundary_changed {
        debug::load_space_boundaries_from_path(
            &watcher.debug_space_boundary_path,
            &mut space_boundaries,
        );
    }
    if nfp_changed {
        debug::load_nfp_from_path(&watcher.debug_nfp_path, &mut nfp_data);
    }
    if ifp_changed {
        debug::load_ifp_from_path(&watcher.debug_ifp_path, &mut ifp_data);
    }

    if boundary_changed {
        debug::load_debug_boundary_from_path(&watcher.debug_boundary_path, &mut boundary_points);
    }
}
