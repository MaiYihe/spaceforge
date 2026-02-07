use bevy::math::vec3;
use bevy::prelude::Vec3;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct ObjBounds {
    pub min: Vec3,
    pub max: Vec3,
}

pub fn load_obj_bounds(path: &str, scale: f32) -> Result<ObjBounds, String> {
    let file = File::open(path).map_err(|e| format!("open obj failed: {e}"))?;
    let reader = BufReader::new(file);

    let mut min = vec3(0.0, 0.0, 0.0);
    let mut max = vec3(0.0, 0.0, 0.0);
    let mut has_bounds = false;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read obj failed: {e}"))?;
        if line.starts_with("v ") {
            let mut it = line.split_whitespace();
            let _ = it.next();
            let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let z: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let v = vec3(x * scale, y * scale, z * scale);
            if !has_bounds {
                min = v;
                max = v;
                has_bounds = true;
            } else {
                min.x = min.x.min(v.x);
                min.y = min.y.min(v.y);
                min.z = min.z.min(v.z);
                max.x = max.x.max(v.x);
                max.y = max.y.max(v.y);
                max.z = max.z.max(v.z);
            }
        }
    }

    if !has_bounds {
        return Err("obj has no vertices".to_string());
    }

    Ok(ObjBounds { min, max })
}

pub fn load_obj_mesh(path: &str, scale: f32) -> Result<Mesh, String> {
    let file = File::open(path).map_err(|e| format!("open obj failed: {e}"))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read obj failed: {e}"))?;
        if line.starts_with("v ") {
            let mut it = line.split_whitespace();
            let _ = it.next();
            let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let z: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            positions.push([x * scale, y * scale, z * scale]);
        } else if line.starts_with("f ") {
            let mut it = line.split_whitespace();
            let _ = it.next();
            let mut face: Vec<i32> = Vec::new();
            for token in it {
                let mut ts = token.split('/');
                if let Some(idx_str) = ts.next() {
                    if idx_str.is_empty() {
                        continue;
                    }
                    let idx: i32 = idx_str.parse().unwrap_or(0);
                    if idx == 0 {
                        continue;
                    }
                    let v = if idx > 0 {
                        idx - 1
                    } else {
                        positions.len() as i32 + idx
                    };
                    face.push(v);
                }
            }
            if face.len() >= 3 {
                for i in 1..face.len() - 1 {
                    indices.push(face[0] as u32);
                    indices.push(face[i] as u32);
                    indices.push(face[i + 1] as u32);
                }
            }
        }
    }

    if positions.is_empty() || indices.is_empty() {
        return Err("obj has no vertices or faces".to_string());
    }

    let normals = compute_vertex_normals(&positions, &indices);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
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
