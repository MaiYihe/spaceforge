use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use types::{RegionsType, RegionsTypeMask};
use usd_core::{SpaceUsd, UsdMesh};

#[derive(Debug, Clone, Copy)]
pub struct Bounds3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct RegionsTypeConfig {
    regions_types: Vec<RegionsTypeItem>,
}

#[derive(Debug, Deserialize)]
struct RegionsTypeItem {
    id: RegionsType,
    name: String,
}

pub fn load_regions_type_registry(path: &str) -> Result<HashMap<String, RegionsType>, String> {
    let data = fs::read_to_string(path)
        .map_err(|e| format!("read regions types config failed ({path}): {e}"))?;
    let config: RegionsTypeConfig =
        toml::from_str(&data).map_err(|e| format!("parse regions types config failed: {e}"))?;

    let mut map = HashMap::new();
    for item in config.regions_types {
        map.insert(item.name, item.id);
    }
    Ok(map)
}

pub fn load_bounds(path: &str, scale: f32) -> Result<Bounds3, String> {
    let mesh = load_mesh(path, scale)?;
    if mesh.positions.is_empty() {
        return Err("usda has no vertices".to_string());
    }

    let mut min = mesh.positions[0];
    let mut max = mesh.positions[0];
    for v in &mesh.positions[1..] {
        min[0] = min[0].min(v[0]);
        min[1] = min[1].min(v[1]);
        min[2] = min[2].min(v[2]);
        max[0] = max[0].max(v[0]);
        max[1] = max[1].max(v[1]);
        max[2] = max[2].max(v[2]);
    }

    Ok(Bounds3 { min, max })
}

pub fn load_mesh(path: &str, scale: f32) -> Result<MeshData, String> {
    let meshes = usd_core::load_space_usda(path)?;
    mesh_data_from_scene(&meshes, scale)
}

pub(crate) fn mesh_data_from_scene(
    scene: &[SpaceUsd],
    scale: f32,
) -> Result<MeshData, String> {
    let mut merged_positions = Vec::<[f32; 3]>::new();
    let mut merged_indices = Vec::<u32>::new();

    for mesh in scene {
        let local = to_mesh_data_from_usd(&mesh.mesh, scale)?;
        append_mesh_data(&mut merged_positions, &mut merged_indices, &local);
    }

    if merged_positions.is_empty() || merged_indices.is_empty() {
        return Err("usda has no triangle mesh data".to_string());
    }

    Ok(MeshData {
        positions: merged_positions,
        indices: merged_indices,
    })
}

pub(crate) fn to_mesh_data_from_usd(
    mesh: &UsdMesh,
    scale: f32,
) -> Result<MeshData, String> {
    let positions: Vec<[f32; 3]> = mesh
        .mesh
        .points
        .iter()
        .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
        .collect();

    let mut indices = Vec::<u32>::new();
    let mut cursor = 0usize;

    for &count in &mesh.mesh.counts {
        let count = count as usize;
        if count < 3 {
            cursor = cursor.saturating_add(count);
            continue;
        }
        if cursor + count > mesh.mesh.indices.len() {
            return Err(format!(
                "mesh '{}' has inconsistent face counts/indices",
                mesh.path
            ));
        }

        let base = mesh.mesh.indices[cursor] as usize;
        for i in 1..count - 1 {
            let i1 = mesh.mesh.indices[cursor + i] as usize;
            let i2 = mesh.mesh.indices[cursor + i + 1] as usize;
            if base >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
                return Err(format!("mesh '{}' has out-of-range face index", mesh.path));
            }
            indices.push(base as u32);
            indices.push(i1 as u32);
            indices.push(i2 as u32);
        }

        cursor += count;
    }

    if positions.is_empty() || indices.is_empty() {
        return Err(format!("mesh '{}' has no triangles", mesh.path));
    }

    Ok(MeshData { positions, indices })
}

fn append_mesh_data(
    merged_positions: &mut Vec<[f32; 3]>,
    merged_indices: &mut Vec<u32>,
    local: &MeshData,
) {
    let base = merged_positions.len() as u32;
    merged_positions.extend(local.positions.iter().copied());
    for idx in &local.indices {
        merged_indices.push(base + *idx);
    }
}

pub(crate) fn mask_from_names(
    names: &[String],
    regions_type_ids: &HashMap<String, RegionsType>,
) -> Result<RegionsTypeMask, String> {
    let mut mask = RegionsTypeMask::NONE;
    for name in names {
        let id = regions_type_ids
            .get(name)
            .ok_or_else(|| format!("regions type '{name}' not found in registry"))?;
        if !mask.insert_id(*id) {
            return Err(format!("regions type id out of range [0,31]: {id}"));
        }
    }
    Ok(mask)
}
