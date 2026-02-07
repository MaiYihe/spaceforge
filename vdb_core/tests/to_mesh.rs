mod common;

use vdb_core::VdbGrid;

#[test]
#[ignore]
fn to_mesh_creates_geometry() {
    VdbGrid::init();
    let path = common::load_first_obj_path().expect("failed to load obj_path from config");
    let grid = VdbGrid::from_obj_path(&path, 15.0, 1000.0)
        .expect("failed to create grid from obj");
    let mesh = grid.to_mesh(0.0, 0.0).expect("failed to generate mesh");
    assert!(!mesh.positions.is_empty(), "mesh has no vertices");
    assert!(!mesh.indices.is_empty(), "mesh has no indices");
}
