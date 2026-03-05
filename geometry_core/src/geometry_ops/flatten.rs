use crate::models::mesh::Mesh;

/// Flattens a mesh onto the XZ plane (y=0) and returns all vertices.
pub fn flatten_to_xz_points(mesh: &Mesh) -> Vec<[f32; 3]> {
    mesh.positions
        .iter()
        .map(|p| [p[0], 0.0, p[2]])
        .collect()
}
