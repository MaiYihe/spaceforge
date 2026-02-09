use std::fs::File;
use std::io::{BufRead, BufReader};

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

pub fn load_obj_bounds(path: &str, scale: f32) -> Result<Bounds3, String> {
    let file = File::open(path).map_err(|e| format!("open obj failed: {e}"))?;
    let reader = BufReader::new(file);

    let mut min = [0.0f32; 3];
    let mut max = [0.0f32; 3];
    let mut has_bounds = false;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read obj failed: {e}"))?;
        if line.starts_with("v ") {
            let mut it = line.split_whitespace();
            let _ = it.next();
            let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let z: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let v = [x * scale, y * scale, z * scale];
            if !has_bounds {
                min = v;
                max = v;
                has_bounds = true;
            } else {
                min[0] = min[0].min(v[0]);
                min[1] = min[1].min(v[1]);
                min[2] = min[2].min(v[2]);
                max[0] = max[0].max(v[0]);
                max[1] = max[1].max(v[1]);
                max[2] = max[2].max(v[2]);
            }
        }
    }

    if !has_bounds {
        return Err("obj has no vertices".to_string());
    }

    Ok(Bounds3 { min, max })
}

pub fn load_obj_mesh(path: &str, scale: f32) -> Result<MeshData, String> {
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
                    let v = if idx > 0 { idx - 1 } else { positions.len() as i32 + idx };
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

    Ok(MeshData { positions, indices })
}
