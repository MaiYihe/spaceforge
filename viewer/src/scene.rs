use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use std::hash::{Hash, Hasher};

use crate::camera::OrbitCamera;
use vdb_core::VdbGrid;
use crate::ui::{BBoxEntity, BBoxTextEntity, SceneInfo};
use crate::config::{ViewMode, ViewerConfig};
use crate::vdb_mesh::mesh_from_vdb;
use crate::voxel_instancing::{spawn_voxel_instances, VoxelInstance};
use asset_import::{load_bounds, load_mesh};
use std::collections::HashSet;

impl Hash for crate::config::Pose2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.theta.to_bits().hash(state);
    }
}

impl Hash for crate::config::PlacementConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.obj_path.hash(state);
        self.pose.hash(state);
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<ViewerConfig>,
) {
    let mode = config.mode;
    if matches!(mode, ViewMode::Sdf | ViewMode::Voxel) {
        VdbGrid::init();
    }

    let obj_scale = config.obj_scale; // meters -> millimeters
    let voxel_size = config.voxel_size; // mm voxels
    let placements = &config.assets;

    let mut combined_min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut combined_max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

    let mut solid_instances: Vec<VoxelInstance> = Vec::new();
    let mut line_instances: Vec<VoxelInstance> = Vec::new();
    let cube_solid = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let cube_line = meshes.add(cube_line_mesh());

    for placement in placements {
        let obj_bounds = load_bounds(&placement.obj_path, obj_scale)
            .unwrap_or_else(|_| panic!("Failed to load mesh bounds: {}", placement.obj_path));

        let translation = Vec3::new(placement.pose.x, 0.0, placement.pose.y);
        let rotation = Quat::from_rotation_y(placement.pose.theta);

        let min = Vec3::from(obj_bounds.min) + translation;
        let max = Vec3::from(obj_bounds.max) + translation;
        combined_min.x = combined_min.x.min(min.x);
        combined_min.y = combined_min.y.min(min.y);
        combined_min.z = combined_min.z.min(min.z);
        combined_max.x = combined_max.x.max(max.x);
        combined_max.y = combined_max.y.max(max.y);
        combined_max.z = combined_max.z.max(max.z);

        match mode {
            ViewMode::Voxel => {
                let grid = VdbGrid::from_obj_path(&placement.obj_path, voxel_size, obj_scale)
                    .unwrap_or_else(|_| panic!("Failed to load OBJ: {}", placement.obj_path));

                let voxel_size = grid.voxel_size();
                let coords = grid
                    .active_voxel_coords()
                    .unwrap_or_else(|_| panic!("Failed to fetch voxels: {}", placement.obj_path));

                let mut set: HashSet<(i32, i32, i32)> = HashSet::with_capacity(coords.len());
                for c in &coords {
                    set.insert((c[0], c[1], c[2]));
                }

                for c in coords {
                    let (x, y, z) = (c[0], c[1], c[2]);
                    let neighbors = [
                        (x + 1, y, z),
                        (x - 1, y, z),
                        (x, y + 1, z),
                        (x, y - 1, z),
                        (x, y, z + 1),
                        (x, y, z - 1),
                    ];
                    let mut is_surface = false;
                    for n in neighbors {
                        if !set.contains(&n) {
                            is_surface = true;
                            break;
                        }
                    }
                    if !is_surface {
                        continue;
                    }

                    let world = Vec3::new(x as f32, y as f32, z as f32) * voxel_size;
                    let v = rotation * world + translation;
                    solid_instances.push(VoxelInstance {
                        position: v,
                        scale: voxel_size,
                        color: [1.0, 1.0, 1.0, 1.0],
                    });
                    line_instances.push(VoxelInstance {
                        position: v,
                        scale: voxel_size * 1.001,
                        color: [0.0, 0.0, 0.0, 1.0],
                    });
                }
            }
            ViewMode::Sdf => {
                let grid = VdbGrid::from_obj_path(&placement.obj_path, voxel_size, obj_scale)
                    .unwrap_or_else(|_| panic!("Failed to load OBJ: {}", placement.obj_path));
                let mut mesh = mesh_from_vdb(&grid, 0.0, 0.0)
                    .unwrap_or_else(|_| panic!("Failed to mesh OBJ: {}", placement.obj_path));
                apply_transform_to_mesh(&mut mesh, rotation, translation);
                let material = materials.add(StandardMaterial {
                    base_color: Color::rgb(0.97, 0.97, 0.97),
                    perceptual_roughness: 0.6,
                    metallic: 0.0,
                    ..default()
                });
                commands.spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material,
                    ..default()
                });
            }
            ViewMode::Obj => {
                let mut mesh = mesh_from_asset(&placement.obj_path, obj_scale)
                    .unwrap_or_else(|_| panic!("Failed to load mesh: {}", placement.obj_path));
                apply_transform_to_mesh(&mut mesh, rotation, translation);
                let material = materials.add(StandardMaterial {
                    base_color: Color::rgb(0.97, 0.97, 0.97),
                    perceptual_roughness: 0.6,
                    metallic: 0.0,
                    ..default()
                });
                commands.spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material,
                    ..default()
                });
            }
        }
    }

    let center = (combined_min + combined_max) * 0.5;
    let extent = combined_max - combined_min;
    let size = vec3(extent.x.abs(), extent.y.abs(), extent.z.abs());
    let max_extent = size.x.max(size.y).max(size.z);
    let focus_distance = (max_extent * 1.5).max(100.0);

    if matches!(mode, ViewMode::Voxel) {
        spawn_voxel_instances(&mut commands, cube_solid, solid_instances);
        spawn_voxel_instances(&mut commands, cube_line, line_instances);
    }

    // BBox
    let bbox_mesh = meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)));
    let bbox_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.0, 0.0, 0.2),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let bbox_entity = commands
        .spawn(PbrBundle {
            mesh: bbox_mesh,
            material: bbox_material,
            transform: Transform::from_translation(center),
            visibility: Visibility::Hidden,
            ..default()
        })
        .id();

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
        size,
    });
    commands.insert_resource(BBoxEntity(bbox_entity));

    // UI text (top-left)
    let mut text_bundle = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 18.0,
            color: Color::RED,
            ..default()
        },
    );
    text_bundle.style = Style {
        position_type: PositionType::Absolute,
        left: Val::Px(12.0),
        top: Val::Px(12.0),
        ..default()
    };
    text_bundle.visibility = Visibility::Hidden;
    let text_entity = commands.spawn(text_bundle).id();
    commands.insert_resource(BBoxTextEntity(text_entity));
}

fn apply_transform_to_mesh(mesh: &mut Mesh, rotation: Quat, translation: Vec3) {
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for p in positions.iter_mut() {
            let v = Vec3::from(*p);
            let v = rotation * v + translation;
            *p = [v.x, v.y, v.z];
        }
    }
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for n in normals.iter_mut() {
            let v = Vec3::from(*n);
            let v = rotation * v;
            *n = [v.x, v.y, v.z];
        }
    }
}

fn mesh_from_asset(path: &str, scale: f32) -> Result<Mesh, String> {
    let data = load_mesh(path, scale)?;
    let normals = compute_vertex_normals(&data.positions, &data.indices);
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(data.indices));
    Ok(mesh)
}

fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32, 0.0f32, 0.0f32]; positions.len()];

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

fn cube_line_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );

    // 8 corners of a unit cube centered at origin.
    let positions = vec![
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [0.5, 0.5, -0.5],   // 2
        [-0.5, 0.5, -0.5],  // 3
        [-0.5, -0.5, 0.5],  // 4
        [0.5, -0.5, 0.5],   // 5
        [0.5, 0.5, 0.5],    // 6
        [-0.5, 0.5, 0.5],   // 7
    ];

    // 12 edges of the cube (line list pairs).
    let indices: Vec<u32> = vec![
        0, 1, 1, 2, 2, 3, 3, 0, // bottom
        4, 5, 5, 6, 6, 7, 7, 4, // top
        0, 4, 1, 5, 2, 6, 3, 7, // sides
    ];

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}
