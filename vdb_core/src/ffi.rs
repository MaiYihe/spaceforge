use std::ffi::c_void;

pub type Grid = c_void;

extern "C" {
    pub(crate) fn vdb_init();
    pub(crate) fn create_from_obj(path: *const i8, voxel_size: f32, scale: f32) -> *mut Grid;
    pub(crate) fn vdb_mesh_from_grid(
        grid: *mut Grid,
        isovalue: f32,
        adaptivity: f32,
        out_vertices: *mut *mut f32,
        out_vertex_count: *mut i32,
        out_indices: *mut *mut i32,
        out_index_count: *mut i32,
    ) -> i32;
    pub(crate) fn vdb_mesh_free(vertices: *mut f32, indices: *mut i32);
    pub(crate) fn vdb_voxel_size(grid: *mut Grid) -> f32;
    pub(crate) fn vdb_active_voxel_centers(
        grid: *mut Grid,
        out_positions: *mut *mut f32,
        out_count: *mut i32,
    ) -> i32;
    pub(crate) fn vdb_active_voxel_centers_free(positions: *mut f32);
    pub(crate) fn vdb_active_voxel_coords(
        grid: *mut Grid,
        out_coords: *mut *mut i32,
        out_count: *mut i32,
    ) -> i32;
    pub(crate) fn vdb_active_voxel_coords_free(coords: *mut i32);
}
