mod common;

use vdb_core::VdbGrid;

#[test]
#[ignore]
fn from_obj_path_creates_grid() {
    VdbGrid::init();
    let path = common::load_first_obj_path().expect("failed to load obj_path from config");
    let grid = VdbGrid::from_obj_path(&path, 15.0, 1000.0)
        .expect("failed to create grid from obj");
    let _ = grid;
}
