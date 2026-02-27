use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use types::{RegionsTypeId, RegionsTypeMask};

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

#[derive(Debug,  Deserialize)]
struct RegionsTypeConfig {
    regions_types: Vec<RegionsTypeItem>,
}

#[derive(Debug, Deserialize)]
struct RegionsTypeItem {
    id: RegionsTypeId,
    name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedMesh {
    pub name: String,
    pub points: Vec<[f32; 3]>,
    pub face_counts: Vec<usize>,
    pub face_indices: Vec<usize>,
    pub regions_type_names : Vec<String>,
}

pub fn load_regions_type_registry(path: &str) -> Result<HashMap<String, RegionsTypeId>, String> {
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
    ensure_usda(path)?;
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
    ensure_usda(path)?;
    let data = fs::read_to_string(path).map_err(|e| format!("read usda failed: {e}"))?;
    let meshes = parse_usda_meshes(&data)?;

    let mut merged_positions = Vec::<[f32; 3]>::new();
    let mut merged_indices = Vec::<u32>::new();

    for mesh in meshes {
        let local = to_mesh_data(&mesh, scale)?;
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

pub(crate) fn ensure_usda(path: &str) -> Result<(), String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .ok_or_else(|| format!("path has no extension: {path}"))?;
    if ext != "usda" {
        return Err(format!("unsupported format .{ext}; only .usda is supported"));
    }
    Ok(())
}

pub(crate) fn extract_first_xform_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def Xform ") {
            continue;
        }
        let start = trimmed.find('"')? + 1;
        let remain = &trimmed[start..];
        let end_rel = remain.find('"')?;
        let name = &remain[..end_rel];
        if name != "HoudiniLayerInfo" {
            return Some(name.to_string());
        }
    }
    None
}

pub(crate) fn parse_usda_meshes(content: &str) -> Result<Vec<ParsedMesh>, String> {
    let mut out = Vec::<ParsedMesh>::new();
    let mut cursor = 0usize;

    while let Some(rel) = content[cursor..].find("def Mesh \"") {
        let mesh_decl_start = cursor + rel;
        let name_start = mesh_decl_start + "def Mesh \"".len();
        let Some(name_end_rel) = content[name_start..].find('"') else {
            return Err("malformed mesh declaration".to_string());
        };
        let name_end = name_start + name_end_rel;
        let name = content[name_start..name_end].to_string();

        let Some(brace_start_rel) = content[name_end..].find('{') else {
            return Err(format!("mesh '{name}' has no opening brace"));
        };
        let brace_start = name_end + brace_start_rel;
        let brace_end = find_matching_brace(content, brace_start)
            .ok_or_else(|| format!("mesh '{name}' has unclosed brace"))?;
        let block = &content[brace_start + 1..brace_end];

        let points_src = extract_bracketed_after(block, "point3f[] points")
            .ok_or_else(|| format!("mesh '{name}' missing points"))?;
        let counts_src = extract_bracketed_after(block, "int[] faceVertexCounts")
            .ok_or_else(|| format!("mesh '{name}' missing faceVertexCounts"))?;
        let indices_src = extract_bracketed_after(block, "int[] faceVertexIndices")
            .ok_or_else(|| format!("mesh '{name}' missing faceVertexIndices"))?;
        let mask_src = extract_bracketed_after(block, "custom string[] regionsTypeMask");

        let points = parse_points(&points_src)?;
        let face_counts = parse_usize_list(&counts_src)?;
        let face_indices = parse_usize_list(&indices_src)?;
        let regions_type_names = if let Some(raw) = mask_src {
            parse_string_list(&raw)
        } else {
            Vec::new()
        };

        out.push(ParsedMesh {
            name,
            points,
            face_counts,
            face_indices,
            regions_type_names,
        });

        cursor = brace_end + 1;
    }

    if out.is_empty() {
        return Err("usda contains no Mesh blocks".to_string());
    }

    Ok(out)
}

fn find_matching_brace(s: &str, open_idx: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = open_idx;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

fn extract_bracketed_after(haystack: &str, needle: &str) -> Option<String> {
    let start = haystack.find(needle)?;
    let rhs = &haystack[start + needle.len()..];
    let eq = rhs.find('=')?;
    let rhs = &rhs[eq + 1..];
    let open = rhs.find('[')?;
    let mut depth = 0usize;
    let mut end = None;

    for (i, ch) in rhs.char_indices().skip(open) {
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if depth == 0 {
                end = Some(i);
                break;
            }
        }
    }

    let end = end?;
    Some(rhs[open + 1..end].to_string())
}

fn parse_points(src: &str) -> Result<Vec<[f32; 3]>, String> {
    let mut points = Vec::<[f32; 3]>::new();
    let mut cursor = 0usize;

    while let Some(open_rel) = src[cursor..].find('(') {
        let open = cursor + open_rel;
        let Some(close_rel) = src[open + 1..].find(')') else {
            return Err("invalid point tuple in points array".to_string());
        };
        let close = open + 1 + close_rel;
        let tuple = &src[open + 1..close];
        let vals: Vec<&str> = tuple.split(',').map(|v| v.trim()).collect();
        if vals.len() != 3 {
            return Err("point tuple is not float3".to_string());
        }
        let x: f32 = vals[0].parse().map_err(|_| "invalid x in points".to_string())?;
        let y: f32 = vals[1].parse().map_err(|_| "invalid y in points".to_string())?;
        let z: f32 = vals[2].parse().map_err(|_| "invalid z in points".to_string())?;
        points.push([x, y, z]);
        cursor = close + 1;
    }

    if points.is_empty() {
        return Err("points array is empty".to_string());
    }

    Ok(points)
}

fn parse_usize_list(src: &str) -> Result<Vec<usize>, String> {
    let mut out = Vec::<usize>::new();
    for token in src
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|t| !t.is_empty())
    {
        let n: isize = token
            .parse()
            .map_err(|_| format!("invalid integer token: {token}"))?;
        if n < 0 {
            return Err(format!("negative index/count is unsupported in USDA parser: {n}"));
        }
        out.push(n as usize);
    }
    Ok(out)
}

fn parse_string_list(src: &str) -> Vec<String> {
    let mut out = Vec::<String>::new();
    let mut cursor = 0usize;
    while let Some(start_rel) = src[cursor..].find('"') {
        let start = cursor + start_rel + 1;
        let Some(end_rel) = src[start..].find('"') else {
            break;
        };
        let end = start + end_rel;
        out.push(src[start..end].to_string());
        cursor = end + 1;
    }
    out
}

pub(crate) fn to_mesh_data(mesh: &ParsedMesh, scale: f32) -> Result<MeshData, String> {
    let positions: Vec<[f32; 3]> = mesh
        .points
        .iter()
        .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
        .collect();

    let mut indices = Vec::<u32>::new();
    let mut cursor = 0usize;

    for &count in &mesh.face_counts {
        if count < 3 {
            cursor = cursor.saturating_add(count);
            continue;
        }
        if cursor + count > mesh.face_indices.len() {
            return Err(format!(
                "mesh '{}' has inconsistent face counts/indices",
                mesh.name
            ));
        }

        let base = mesh.face_indices[cursor];
        for i in 1..count - 1 {
            let i1 = mesh.face_indices[cursor + i];
            let i2 = mesh.face_indices[cursor + i + 1];
            if base >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
                return Err(format!("mesh '{}' has out-of-range face index", mesh.name));
            }
            indices.push(base as u32);
            indices.push(i1 as u32);
            indices.push(i2 as u32);
        }

        cursor += count;
    }

    if positions.is_empty() || indices.is_empty() {
        return Err(format!("mesh '{}' has no triangles", mesh.name));
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
    regions_type_ids: &HashMap<String, RegionsTypeId>,
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
