#[doc(hidden)]
mod ffi;

use std::ptr::NonNull;

type Grid = ffi::Grid;
use std::ffi::CString;

#[derive(Debug)]
pub struct VdbGrid {
    raw: NonNull<Grid>,
}

#[derive(Debug, Clone)]
pub struct VdbMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl VdbGrid {
    pub fn init() {
        unsafe { ffi::vdb_init() }
    }

    pub fn from_obj_path(path: &str, voxel_size: f32, scale: f32) -> Result<Self, String> {
        let c_path = CString::new(path).map_err(|_| "obj path contains NUL byte".to_string())?;
        let raw = unsafe { ffi::create_from_obj(c_path.as_ptr(), voxel_size, scale) };
        unsafe { Self::from_raw(raw) }.ok_or_else(|| "failed to create VDB grid".to_string())
    }

    /// # Safety
    /// `raw` must be a valid pointer returned by OpenVDB FFI.
    pub(crate) unsafe fn from_raw(raw: *mut Grid) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw })
    }

    pub(crate) fn as_ptr(&self) -> *mut Grid {
        self.raw.as_ptr()
    }

    pub fn to_mesh(&self, isovalue: f32, adaptivity: f32) -> Result<VdbMesh, String> {
        let mut vertices_ptr: *mut f32 = std::ptr::null_mut();
        let mut indices_ptr: *mut i32 = std::ptr::null_mut();
        let mut vertex_count: i32 = 0;
        let mut index_count: i32 = 0;

        let ok = unsafe {
            ffi::vdb_mesh_from_grid(
                self.as_ptr(),
                isovalue,
                adaptivity,
                &mut vertices_ptr,
                &mut vertex_count,
                &mut indices_ptr,
                &mut index_count,
            )
        };

        if ok == 0 || vertices_ptr.is_null() || indices_ptr.is_null() {
            return Err("failed to extract mesh from VDB grid".to_string());
        }

        if vertex_count <= 0 || index_count <= 0 {
            unsafe { ffi::vdb_mesh_free(vertices_ptr, indices_ptr) };
            return Err("VDB mesh has no vertices or indices".to_string());
        }

        let vertex_len = (vertex_count as usize) * 3;
        let index_len = index_count as usize;

        let vertices = unsafe { std::slice::from_raw_parts(vertices_ptr, vertex_len) };
        let indices = unsafe { std::slice::from_raw_parts(indices_ptr, index_len) };

        let mut positions = Vec::with_capacity(vertex_count as usize);
        for i in 0..vertex_count as usize {
            let base = i * 3;
            positions.push([vertices[base], vertices[base + 1], vertices[base + 2]]);
        }

        let mut out_indices = Vec::with_capacity(index_len);
        for &idx in indices {
            if idx < 0 {
                unsafe { ffi::vdb_mesh_free(vertices_ptr, indices_ptr) };
                return Err("VDB mesh contains negative index".to_string());
            }
            out_indices.push(idx as u32);
        }

        unsafe { ffi::vdb_mesh_free(vertices_ptr, indices_ptr) };

        Ok(VdbMesh {
            positions,
            indices: out_indices,
        })
    }

    pub fn voxel_size(&self) -> f32 {
        unsafe { ffi::vdb_voxel_size(self.as_ptr()) }
    }

    pub fn active_voxel_centers(&self) -> Result<Vec<[f32; 3]>, String> {
        let mut positions_ptr: *mut f32 = std::ptr::null_mut();
        let mut count: i32 = 0;
        let ok = unsafe { ffi::vdb_active_voxel_centers(self.as_ptr(), &mut positions_ptr, &mut count) };
        if ok == 0 || positions_ptr.is_null() || count <= 0 {
            return Err("failed to fetch active voxel centers".to_string());
        }

        let len = (count as usize) * 3;
        let positions = unsafe { std::slice::from_raw_parts(positions_ptr, len) };
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let base = i * 3;
            out.push([positions[base], positions[base + 1], positions[base + 2]]);
        }
        unsafe { ffi::vdb_active_voxel_centers_free(positions_ptr) };
        Ok(out)
    }

    pub fn active_voxel_coords(&self) -> Result<Vec<[i32; 3]>, String> {
        let mut coords_ptr: *mut i32 = std::ptr::null_mut();
        let mut count: i32 = 0;
        let ok = unsafe { ffi::vdb_active_voxel_coords(self.as_ptr(), &mut coords_ptr, &mut count) };
        if ok == 0 || coords_ptr.is_null() || count <= 0 {
            return Err("failed to fetch active voxel coords".to_string());
        }

        let len = (count as usize) * 3;
        let coords = unsafe { std::slice::from_raw_parts(coords_ptr, len) };
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let base = i * 3;
            out.push([coords[base], coords[base + 1], coords[base + 2]]);
        }
        unsafe { ffi::vdb_active_voxel_coords_free(coords_ptr) };
        Ok(out)
    }
}

// NOTE: We do not implement Drop because there is no FFI release function yet.
