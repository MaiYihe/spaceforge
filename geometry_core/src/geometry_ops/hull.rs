use crate::models::mesh::Mesh;
use geo::algorithm::convex_hull::ConvexHull;
use geo::CoordsIter;
use geo_types::{Coord, MultiPoint};

/// Projects all vertices onto the XZ plane and returns the convex hull boundary (outermost ring).
pub fn convex_hull_xz(mesh: &Mesh) -> Vec<[f32; 3]> {
    if mesh.positions.len() < 3 {
        return Vec::new();
    }
    let points: Vec<Coord<f64>> = mesh
        .positions
        .iter()
        .map(|p| Coord {
            x: p[0] as f64,
            y: p[2] as f64,
        })
        .collect();
    let multipoint = MultiPoint::from(points);
    let poly = multipoint.convex_hull();
    let mut out = Vec::new();
    for coord in poly.exterior().coords_iter() {
        out.push([coord.x as f32, 0.0, coord.y as f32]);
    }
    if out.len() > 1 {
        out.pop();
    }
    out
}
