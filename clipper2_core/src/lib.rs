use std::ffi::c_int;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointI64 {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone)]
pub struct PolygonI64 {
    pub outer: Vec<PointI64>,
    pub holes: Vec<Vec<PointI64>>,
}

#[derive(Debug, Clone)]
pub struct MultiPolygonI64 {
    pub polygons: Vec<PolygonI64>,
}

#[derive(Debug)]
pub enum ClipError {
    NotAvailable,
    ComputeFailed(i32),
}

extern "C" {
    fn clipper2_nfp(
        subject_xy: *const i64,
        subject_len: usize,
        subject_offsets: *const i64,
        subject_offset_len: usize,
        clip_xy: *const i64,
        clip_len: usize,
        clip_offsets: *const i64,
        clip_offset_len: usize,
        out_xy: *mut *mut i64,
        out_len: *mut usize,
        out_offsets: *mut *mut i64,
        out_offset_len: *mut usize,
    ) -> c_int;

    fn clipper2_ifp(
        subject_xy: *const i64,
        subject_len: usize,
        subject_offsets: *const i64,
        subject_offset_len: usize,
        container_xy: *const i64,
        container_len: usize,
        container_offsets: *const i64,
        container_offset_len: usize,
        out_xy: *mut *mut i64,
        out_len: *mut usize,
        out_offsets: *mut *mut i64,
        out_offset_len: *mut usize,
    ) -> c_int;

    fn clipper2_free(ptr: *mut i64);
}

pub fn compute_nfp(subject: &MultiPolygonI64, clip: &MultiPolygonI64) -> Result<MultiPolygonI64, ClipError> {
    let (subject_xy, subject_offsets) = flatten(subject);
    let (clip_xy, clip_offsets) = flatten(clip);
    let mut out_xy: *mut i64 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let mut out_offsets: *mut i64 = std::ptr::null_mut();
    let mut out_offset_len: usize = 0;

    let code = unsafe {
        clipper2_nfp(
            subject_xy.as_ptr(),
            subject_xy.len(),
            subject_offsets.as_ptr(),
            subject_offsets.len(),
            clip_xy.as_ptr(),
            clip_xy.len(),
            clip_offsets.as_ptr(),
            clip_offsets.len(),
            &mut out_xy,
            &mut out_len,
            &mut out_offsets,
            &mut out_offset_len,
        )
    };

    if code == 1 {
        return Err(ClipError::NotAvailable);
    }
    if code != 0 {
        return Err(ClipError::ComputeFailed(code));
    }

    let result = unsafe { unflatten(out_xy, out_len, out_offsets, out_offset_len) };
    unsafe {
        if !out_xy.is_null() { clipper2_free(out_xy); }
        if !out_offsets.is_null() { clipper2_free(out_offsets); }
    }
    Ok(result)
}

pub fn compute_ifp(subject: &MultiPolygonI64, container: &MultiPolygonI64) -> Result<MultiPolygonI64, ClipError> {
    let (subject_xy, subject_offsets) = flatten(subject);
    let (container_xy, container_offsets) = flatten(container);
    let mut out_xy: *mut i64 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let mut out_offsets: *mut i64 = std::ptr::null_mut();
    let mut out_offset_len: usize = 0;

    let code = unsafe {
        clipper2_ifp(
            subject_xy.as_ptr(),
            subject_xy.len(),
            subject_offsets.as_ptr(),
            subject_offsets.len(),
            container_xy.as_ptr(),
            container_xy.len(),
            container_offsets.as_ptr(),
            container_offsets.len(),
            &mut out_xy,
            &mut out_len,
            &mut out_offsets,
            &mut out_offset_len,
        )
    };

    if code == 1 {
        return Err(ClipError::NotAvailable);
    }
    if code != 0 {
        return Err(ClipError::ComputeFailed(code));
    }

    let result = unsafe { unflatten(out_xy, out_len, out_offsets, out_offset_len) };
    unsafe {
        if !out_xy.is_null() { clipper2_free(out_xy); }
        if !out_offsets.is_null() { clipper2_free(out_offsets); }
    }
    Ok(result)
}

fn flatten(mp: &MultiPolygonI64) -> (Vec<i64>, Vec<i64>) {
    let mut xy = Vec::new();
    let mut offsets = Vec::new();
    let mut cursor: i64 = 0;
    for poly in &mp.polygons {
        offsets.push(cursor);
        cursor += (poly.outer.len() * 2) as i64;
        for p in &poly.outer {
            xy.push(p.x);
            xy.push(p.y);
        }
        for hole in &poly.holes {
            offsets.push(cursor);
            cursor += (hole.len() * 2) as i64;
            for p in hole {
                xy.push(p.x);
                xy.push(p.y);
            }
        }
    }
    (xy, offsets)
}

unsafe fn unflatten(
    out_xy: *const i64,
    out_len: usize,
    out_offsets: *const i64,
    out_offset_len: usize,
) -> MultiPolygonI64 {
    if out_xy.is_null() || out_len == 0 || out_offsets.is_null() || out_offset_len == 0 {
        return MultiPolygonI64 { polygons: Vec::new() };
    }
    let xy = std::slice::from_raw_parts(out_xy, out_len);
    let offsets = std::slice::from_raw_parts(out_offsets, out_offset_len);

    let mut rings: Vec<Vec<PointI64>> = Vec::new();
    let mut i = 0;
    while i < offsets.len() {
        let start = offsets[i] as usize;
        let end = if i + 1 < offsets.len() {
            offsets[i + 1] as usize
        } else {
            xy.len()
        };
        if start >= end || end > xy.len() {
            i += 1;
            continue;
        }
        let ring = &xy[start..end];
        let mut points = Vec::new();
        let mut j = 0;
        while j + 1 < ring.len() {
            points.push(PointI64 { x: ring[j], y: ring[j + 1] });
            j += 2;
        }
        if points.len() >= 3 {
            rings.push(points);
        }
        i += 1;
    }

    let mut outers: Vec<Vec<PointI64>> = Vec::new();
    let mut holes: Vec<Vec<PointI64>> = Vec::new();
    for ring in rings {
        if signed_area_i64(&ring) >= 0 {
            outers.push(ring);
        } else {
            holes.push(ring);
        }
    }

    let mut polygons: Vec<PolygonI64> = outers
        .into_iter()
        .map(|outer| PolygonI64 { outer, holes: Vec::new() })
        .collect();

    for hole in holes {
        let test = hole[0];
        let mut best_idx: Option<usize> = None;
        let mut best_area: i128 = i128::MAX;
        for (idx, poly) in polygons.iter().enumerate() {
            if point_in_ring(test, &poly.outer) {
                let area = signed_area_i64(&poly.outer).abs();
                if area < best_area {
                    best_area = area;
                    best_idx = Some(idx);
                }
            }
        }
        if let Some(idx) = best_idx {
            polygons[idx].holes.push(hole);
        } else {
            polygons.push(PolygonI64 { outer: hole, holes: Vec::new() });
        }
    }

    MultiPolygonI64 { polygons }
}

fn signed_area_i64(points: &[PointI64]) -> i128 {
    let n = points.len();
    if n < 3 {
        return 0;
    }
    let mut sum: i128 = 0;
    for i in 0..n {
        let a = &points[i];
        let b = &points[(i + 1) % n];
        sum += (a.x as i128) * (b.y as i128) - (b.x as i128) * (a.y as i128);
    }
    sum
}

fn point_in_ring(point: PointI64, ring: &[PointI64]) -> bool {
    let n = ring.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = &ring[i];
        let pj = &ring[j];
        let dy = pj.y - pi.y;
        let intersect = if dy == 0 {
            false
        } else {
            ((pi.y > point.y) != (pj.y > point.y))
                && {
                    let x_intersect = (pj.x - pi.x) as f64 * (point.y - pi.y) as f64 / dy as f64
                        + pi.x as f64;
                    (point.x as f64) < x_intersect
                }
        };
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}
