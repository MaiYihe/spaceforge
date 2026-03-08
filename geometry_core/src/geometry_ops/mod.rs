pub mod boundary;
pub mod flatten;
pub mod hull;
pub mod plane;
pub mod sampling;
pub mod nesting;

pub use boundary::{flatten_outer_boundary, mesh_boundary_loop};
pub use flatten::flatten_to_xz_points;
pub use hull::{convex_hull_xz, convex_hull_xz_points};
pub use sampling::{sample_points_area, sample_points_uv};
pub use nesting::{compute_ifp, compute_nfp, MultiPolygonF32, PolygonF32, DEFAULT_SCALE};
