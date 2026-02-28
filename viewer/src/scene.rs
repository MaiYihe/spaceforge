use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;

use crate::camera::OrbitCamera;
use crate::config::ViewerConfig;
use crate::ui::{BBoxEntity, BBoxTextEntity, SceneInfo};
use asset_import::{
    load_bounds, load_placement_region_model_from_usda, load_space_model_from_usda,
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<ViewerConfig>,
) {
    let usda_scale = config.usda_scale;

    let regions_type_ids = asset_import::load_regions_type_registry(&config.regions_type_path)
        .unwrap_or_else(|err| panic!("Failed to load regions type config: {err}"));

    let space = load_space_model_from_usda(
        &config.space_usda_path,
        &regions_type_ids,
        usda_scale,
    )
    .unwrap_or_else(|err| panic!("Failed to load space usda: {err}"));

    let placement = load_placement_region_model_from_usda(
        &config.placement_region_usda_path,
        &regions_type_ids,
        usda_scale,
    )
    .unwrap_or_else(|err| panic!("Failed to load placement region usda: {err}"));

    let space_bounds = load_bounds(&config.space_usda_path, usda_scale)
        .unwrap_or_else(|err| panic!("Failed to load space bounds: {err}"));
    let combined_min = Vec3::from(space_bounds.min);
    let combined_max = Vec3::from(space_bounds.max);

    for mesh in &space.meshes {
        let mesh = mesh_from_geometry(mesh.positions.clone(), mesh.indices.clone());
        let material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.85, 0.85, 0.85),
            perceptual_roughness: 0.7,
            metallic: 0.0,
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            ..default()
        });
    }

    let restricted_mesh = mesh_from_geometry(
        placement.regions.restricted_region.mesh.positions.clone(),
        placement.regions.restricted_region.mesh.indices.clone(),
    );
    let restricted_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.2, 0.6, 1.0, 0.4),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(restricted_mesh),
        material: restricted_material,
        ..default()
    });

    let forbidden_mesh = mesh_from_geometry(
        placement.regions.forbidden_region.mesh.positions.clone(),
        placement.regions.forbidden_region.mesh.indices.clone(),
    );
    let forbidden_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.3, 0.2, 0.4),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(forbidden_mesh),
        material: forbidden_material,
        ..default()
    });

    if !placement.visual.footprint_2d.positions.is_empty()
        && !placement.visual.footprint_2d.indices.is_empty()
    {
        let footprint_mesh = mesh_from_geometry(
            placement.visual.footprint_2d.positions.clone(),
            placement.visual.footprint_2d.indices.clone(),
        );
        let footprint_material = materials.add(StandardMaterial {
            base_color: Color::rgba(0.1, 0.9, 0.5, 0.4),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: meshes.add(footprint_mesh),
            material: footprint_material,
            ..default()
        });
    }

    let center = (combined_min + combined_max) * 0.5;
    let extent = combined_max - combined_min;
    let size = vec3(extent.x.abs(), extent.y.abs(), extent.z.abs());
    let max_extent = size.x.max(size.y).max(size.z);
    let focus_distance = (max_extent * 1.5).max(100.0);

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
