use bevy::math::vec3;
use bevy::prelude::*;

use geometry_core::models::placement_region::SdfGrid;

use super::{PlacementRenderMode, SceneEntities, SceneInfo, ScenePayload, SceneTransforms};
use crate::camera::OrbitCamera;
use crate::scene::transform::{
    apply_optional_transform, apply_transform_point, apply_transform_positions, compute_bounds,
    find_transform,
};

pub(super) fn apply_payload(
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
    if !scene_info.camera_initialized {
        cam.target = center;
        cam.distance = focus_distance;
        scene_info.camera_initialized = true;
    }

    for (idx, (surface, meta)) in payload
        .space
        .surfaces
        .iter()
        .zip(payload.space.surface_metas.iter())
        .enumerate()
    {
        let mut positions = surface.mesh.positions.clone();
        if let Some(xform) = find_transform(&transforms.space_meshes, idx) {
            apply_transform_positions(&mut positions, xform);
        }
        let mesh = mesh_from_geometry(positions, surface.mesh.indices.clone());
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

pub(super) fn load_render_mode() -> PlacementRenderMode {
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
    sdf: &SdfGrid,
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
