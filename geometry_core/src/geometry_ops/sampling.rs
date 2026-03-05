use crate::models::mesh::Mesh;
use crate::geometry_ops::plane::fit_plane_pca;
use nalgebra::Vector3;

/// Samples points on a (mostly) planar mesh by projecting to UV and
/// taking a grid at `step_mm` (default unit is mm).
pub fn sample_points_uv(mesh: &Mesh, step_mm: f32) -> Vec<[f32; 3]> {
    sample_points_uv_common(&mesh.positions, &mesh.indices, step_mm)
}

fn sample_points_uv_common(
    positions: &[[f32; 3]],
    indices: &[u32],
    step_mm: f32,
) -> Vec<[f32; 3]> {
    if positions.is_empty() || indices.len() < 3 || step_mm <= 0.0 {
        return Vec::new();
    }

    let (origin, u_axis, v_axis, _normal) = match fit_plane_pca(positions) {
        Some(data) => data,
        None => return Vec::new(),
    };

    let mut projected = Vec::with_capacity(positions.len());
    for p in positions {
        let v = Vector3::new(p[0], p[1], p[2]) - origin;
        projected.push([v.dot(&u_axis), v.dot(&v_axis)]);
    }

    let mut min_u = f32::INFINITY;
    let mut max_u = f32::NEG_INFINITY;
    let mut min_v = f32::INFINITY;
    let mut max_v = f32::NEG_INFINITY;
    for uv in &projected {
        min_u = min_u.min(uv[0]);
        max_u = max_u.max(uv[0]);
        min_v = min_v.min(uv[1]);
        max_v = max_v.max(uv[1]);
    }

    if !min_u.is_finite() || !max_u.is_finite() || !min_v.is_finite() || !max_v.is_finite() {
        return Vec::new();
    }

    let mut points = Vec::new();
    let mut u = min_u;
    while u <= max_u + f32::EPSILON {
        let mut v = min_v;
        while v <= max_v + f32::EPSILON {
            if point_in_mesh_2d([u, v], &projected, indices) {
                let p3 = origin + u_axis * u + v_axis * v;
                points.push([p3.x, p3.y, p3.z]);
            }
            v += step_mm;
        }
        u += step_mm;
    }

    points
}

fn point_in_mesh_2d(p: [f32; 2], verts: &[[f32; 2]], indices: &[u32]) -> bool {
    let mut i = 0;
    while i + 2 < indices.len() {
        let a = indices[i] as usize;
        let b = indices[i + 1] as usize;
        let c = indices[i + 2] as usize;
        i += 3;
        if a >= verts.len() || b >= verts.len() || c >= verts.len() {
            continue;
        }
        if point_in_tri_2d(p, verts[a], verts[b], verts[c]) {
            return true;
        }
    }
    false
}

fn point_in_tri_2d(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let v0 = [c[0] - a[0], c[1] - a[1]];
    let v1 = [b[0] - a[0], b[1] - a[1]];
    let v2 = [p[0] - a[0], p[1] - a[1]];

    let dot00 = v0[0] * v0[0] + v0[1] * v0[1];
    let dot01 = v0[0] * v1[0] + v0[1] * v1[1];
    let dot02 = v0[0] * v2[0] + v0[1] * v2[1];
    let dot11 = v1[0] * v1[0] + v1[1] * v1[1];
    let dot12 = v1[0] * v2[0] + v1[1] * v2[1];

    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < f32::EPSILON {
        return false;
    }
    let inv = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv;
    let v = (dot00 * dot12 - dot01 * dot02) * inv;
    u >= 0.0 && v >= 0.0 && u + v <= 1.0
}
