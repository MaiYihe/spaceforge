use clipper2_core::{compute_ifp as ifp_core, compute_nfp as nfp_core, ClipError, MultiPolygonI64, PointI64, PolygonI64};

pub const DEFAULT_SCALE: i64 = 1000; // mm * 1000 => micron precision

/// Computes the No-Fit Polygon (NFP) for subject vs clip.
///
/// Input: subject and clip as multi-polygons (outer + holes) in f32 (XY).
/// Output: multi-polygons in f32 (XY). Supports concave + holes.
pub fn compute_nfp(
    subject: &MultiPolygonF32,
    clip: &MultiPolygonF32,
    scale: i64,
) -> Result<MultiPolygonF32, ClipError> {
    let subject_i = to_i64(subject, scale);
    let clip_i = to_i64(clip, scale);
    let out_i = nfp_core(&subject_i, &clip_i)?;
    Ok(from_i64(&out_i, scale))
}

/// Computes the Inner-Fit Polygon (IFP) for subject inside container.
///
/// Input: subject and container as multi-polygons (outer + holes) in f32 (XY).
/// Output: multi-polygons in f32 (XY). Supports concave + holes.
pub fn compute_ifp(
    subject: &MultiPolygonF32,
    container: &MultiPolygonF32,
    scale: i64,
) -> Result<MultiPolygonF32, ClipError> {
    let subject_i = to_i64(subject, scale);
    let container_i = to_i64(container, scale);
    let out_i = ifp_core(&subject_i, &container_i)?;
    Ok(from_i64(&out_i, scale))
}

#[derive(Debug, Clone)]
pub struct PolygonF32 {
    pub outer: Vec<[f32; 2]>,
    pub holes: Vec<Vec<[f32; 2]>>,
}

#[derive(Debug, Clone)]
pub struct MultiPolygonF32 {
    pub polygons: Vec<PolygonF32>,
}

fn to_i64(mp: &MultiPolygonF32, scale: i64) -> MultiPolygonI64 {
    MultiPolygonI64 {
        polygons: mp
            .polygons
            .iter()
            .map(|p| PolygonI64 {
                outer: normalize_ring(
                    &p.outer
                        .iter()
                        .map(|pt| PointI64 {
                            x: (pt[0] as f64 * scale as f64).round() as i64,
                            y: (pt[1] as f64 * scale as f64).round() as i64,
                        })
                        .collect::<Vec<_>>(),
                    true,
                ),
                holes: p
                    .holes
                    .iter()
                    .map(|hole| {
                        normalize_ring(
                            &hole
                                .iter()
                                .map(|pt| PointI64 {
                                    x: (pt[0] as f64 * scale as f64).round() as i64,
                                    y: (pt[1] as f64 * scale as f64).round() as i64,
                                })
                                .collect::<Vec<_>>(),
                            false,
                        )
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn from_i64(mp: &MultiPolygonI64, scale: i64) -> MultiPolygonF32 {
    let inv = 1.0 / scale as f32;
    MultiPolygonF32 {
        polygons: mp
            .polygons
            .iter()
            .map(|p| PolygonF32 {
                outer: p
                    .outer
                    .iter()
                    .map(|pt| [pt.x as f32 * inv, pt.y as f32 * inv])
                    .collect(),
                holes: p
                    .holes
                    .iter()
                    .map(|hole| {
                        hole.iter()
                            .map(|pt| [pt.x as f32 * inv, pt.y as f32 * inv])
                            .collect()
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn normalize_ring(points: &Vec<PointI64>, outer: bool) -> Vec<PointI64> {
    if points.len() < 3 {
        return points.clone();
    }
    let area = signed_area_i64(points);
    let is_positive = area >= 0;
    if outer && !is_positive {
        let mut rev = points.clone();
        rev.reverse();
        rev
    } else if !outer && is_positive {
        let mut rev = points.clone();
        rev.reverse();
        rev
    } else {
        points.clone()
    }
}

fn signed_area_i64(points: &[PointI64]) -> i128 {
    let mut sum: i128 = 0;
    let n = points.len();
    for i in 0..n {
        let a = &points[i];
        let b = &points[(i + 1) % n];
        sum += (a.x as i128) * (b.y as i128) - (b.x as i128) * (a.y as i128);
    }
    sum
}
