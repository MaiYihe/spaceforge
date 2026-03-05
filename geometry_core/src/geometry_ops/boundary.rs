use crate::geometry_ops::plane::fit_plane_pca;
use crate::models::mesh::Mesh;
use nalgebra::Vector3;
use std::collections::{BTreeMap, HashMap};

/// Flattens a mesh onto its best-fit plane and returns the outer boundary loop vertices.
pub fn flatten_outer_boundary(mesh: &Mesh) -> Vec<[f32; 3]> {
    flatten_outer_boundary_common(&mesh.positions, &mesh.indices)
}

fn flatten_outer_boundary_common(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 3]> {
    if positions.is_empty() || indices.len() < 3 {
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

    let loops = boundary_loops(indices, &projected);
    let Some(loop_indices) = loops
        .into_iter()
        .max_by(|a, b| loop_perimeter(a, &projected).partial_cmp(&loop_perimeter(b, &projected)).unwrap())
    else {
        return Vec::new();
    };

    loop_indices
        .into_iter()
        .map(|idx| {
            let uv = projected[idx];
            let p3 = origin + u_axis * uv[0] + v_axis * uv[1];
            [p3.x, p3.y, p3.z]
        })
        .collect()
}

fn boundary_loops(indices: &[u32], verts: &[[f32; 2]]) -> Vec<Vec<usize>> {
    let mut edge_counts: HashMap<(usize, usize), u32> = HashMap::new();

    let mut i = 0;
    while i + 2 < indices.len() {
        let a = indices[i] as usize;
        let b = indices[i + 1] as usize;
        let c = indices[i + 2] as usize;
        i += 3;
        if a >= verts.len() || b >= verts.len() || c >= verts.len() {
            continue;
        }
        add_edge(a, b, &mut edge_counts);
        add_edge(b, c, &mut edge_counts);
        add_edge(c, a, &mut edge_counts);
    }

    let mut adjacency: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for ((u, v), count) in edge_counts {
        if count == 1 {
            adjacency.entry(u).or_default().push(v);
            adjacency.entry(v).or_default().push(u);
        }
    }

    let mut loops = Vec::new();
    let mut visited_edges: HashMap<(usize, usize), bool> = HashMap::new();

    for (&start, neighbors) in &adjacency {
        for &next in neighbors {
            if visited_edges.get(&(start, next)).copied().unwrap_or(false) {
                continue;
            }
            let mut loop_vertices = Vec::new();
            let mut prev = start;
            let mut curr = next;
            loop_vertices.push(start);

            loop {
                visited_edges.insert((prev, curr), true);
                visited_edges.insert((curr, prev), true);
                loop_vertices.push(curr);

                let next_candidates = adjacency.get(&curr).map(|v| v.as_slice()).unwrap_or(&[]);
                let mut chosen = None;
                for &cand in next_candidates {
                    if cand != prev {
                        chosen = Some(cand);
                        break;
                    }
                }
                let Some(next_v) = chosen else { break; };
                prev = curr;
                curr = next_v;
                if curr == start {
                    break;
                }
                if loop_vertices.len() > verts.len() + 1 {
                    break;
                }
            }

            if loop_vertices.len() >= 3 {
                loops.push(loop_vertices);
            }
        }
    }

    loops
}

fn add_edge(a: usize, b: usize, edge_counts: &mut HashMap<(usize, usize), u32>) {
    let key = if a < b { (a, b) } else { (b, a) };
    *edge_counts.entry(key).or_insert(0) += 1;
}

fn loop_perimeter(indices: &[usize], verts: &[[f32; 2]]) -> f32 {
    if indices.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..indices.len() {
        let a = verts[indices[i]];
        let b = verts[indices[(i + 1) % indices.len()]];
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        sum += (dx * dx + dy * dy).sqrt();
    }
    sum
}
