use crate::config::SceneConfig;
use crate::steps::{derive, import, transform};
use geometry_core::models::placement_region_instance::PlacementTransform;
use geometry_core::geometry_ops::{
    compute_ifp, compute_nfp, DEFAULT_SCALE, MultiPolygonF32, PolygonF32,
};

pub struct StepOutputs {
    pub transforms: Vec<PlacementTransform>,
    pub space_points: Vec<[f32; 3]>,
    pub space_boundaries: Vec<Vec<[f32; 3]>>,
    pub forbidden_points: Vec<[f32; 3]>,
    pub forbidden_hull: Vec<[f32; 3]>,
    pub nfp_rect: Vec<[f32; 3]>,
    pub nfp_result: Vec<Vec<[f32; 3]>>,
    pub ifp_result: Vec<Vec<[f32; 3]>>,
    pub ifp_sample_points: Vec<[f32; 3]>,
    pub ifp_instance_loops: Vec<Vec<[f32; 3]>>,
}

pub fn run_steps(config: &SceneConfig) -> Result<StepOutputs, String> {
    let mut data = import::load_scene_data(config)?;
    transform::attach_transforms(&mut data.instances);

    let transforms: Vec<PlacementTransform> = data
        .instances
        .iter()
        .map(|instance| instance.transform)
        .collect();

    let space_points = derive::derive_space_points(&data.space)?;
    let space_boundaries = derive::derive_space_boundaries(&data.space);

    let derived = derive::derive_forbidden_region(&data.placements, &data.instances)
        .ok_or_else(|| "placement_region_derived skipped (no PlacementRegions)".to_string())?;

    let rect = vec![
        [-500.0, 0.0, -300.0],
        [500.0, 0.0, -300.0],
        [500.0, 0.0, 300.0],
        [-500.0, 0.0, 300.0],
    ];
    let rect_2d: Vec<[f32; 2]> = rect.iter().map(|p| [p[0], p[2]]).collect();
    let hull_2d: Vec<[f32; 2]> = derived.hull.iter().map(|p| [p[0], p[2]]).collect();
    log_polygon_stats("subject(hull)", &hull_2d);
    let subject = MultiPolygonF32 {
        polygons: vec![PolygonF32 {
            outer: rect_2d,
            holes: Vec::new(),
        }],
    };
    let clip = MultiPolygonF32 {
        polygons: vec![PolygonF32 {
            outer: hull_2d,
            holes: Vec::new(),
        }],
    };
    let nfp_mp =
        compute_nfp(&subject, &clip, DEFAULT_SCALE).unwrap_or(MultiPolygonF32 { polygons: Vec::new() });
    let nfp_3d: Vec<Vec<[f32; 3]>> = nfp_mp
        .polygons
        .iter()
        .map(|p| p.outer.iter().map(|pt| [pt[0], 0.0, pt[1]]).collect())
        .collect();

    let ifp_mp = if let Some(surface0) = data.space.surfaces.first() {
        let container_outer: Vec<[f32; 2]> =
            surface0.boundary.iter().map(|p| [p[0], p[2]]).collect();
        log_polygon_stats("container(surface0)", &container_outer);
        let container = MultiPolygonF32 {
            polygons: vec![PolygonF32 {
                outer: container_outer,
                holes: Vec::new(),
            }],
        };
        compute_ifp(&clip, &container, DEFAULT_SCALE)
            .unwrap_or(MultiPolygonF32 { polygons: Vec::new() })
    } else {
        MultiPolygonF32 { polygons: Vec::new() }
    };
    let ifp_3d: Vec<Vec<[f32; 3]>> = ifp_mp
        .polygons
        .iter()
        .map(|p| p.outer.iter().map(|pt| [pt[0], 0.0, pt[1]]).collect())
        .collect();
    log_loops_stats("ifp", &ifp_3d);

    let ifp_sample_points = sample_loop_points(&ifp_3d, 2000.0);
    let ifp_instance_loops = translate_loops(&derived.hull, &ifp_sample_points);

    Ok(StepOutputs {
        transforms,
        space_points,
        space_boundaries,
        forbidden_points: derived.samples,
        forbidden_hull: derived.hull,
        nfp_rect: rect,
        nfp_result: nfp_3d,
        ifp_result: ifp_3d,
        ifp_sample_points,
        ifp_instance_loops,
    })
}

fn log_polygon_stats(label: &str, points: &[[f32; 2]]) {
    if points.is_empty() {
        log::info!("solver::ifp: {label} empty");
        return;
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    for p in points {
        min_x = min_x.min(p[0]);
        min_y = min_y.min(p[1]);
        max_x = max_x.max(p[0]);
        max_y = max_y.max(p[1]);
        sum_x += p[0] as f64;
        sum_y += p[1] as f64;
    }
    let n = points.len() as f64;
    let cx = (sum_x / n) as f32;
    let cy = (sum_y / n) as f32;
    log::info!(
        "solver::ifp: {} count={} bbox=({:.3},{:.3})-({:.3},{:.3}) centroid=({:.3},{:.3})",
        label,
        points.len(),
        min_x,
        min_y,
        max_x,
        max_y,
        cx,
        cy
    );
}

fn log_loops_stats(label: &str, loops: &[Vec<[f32; 3]>]) {
    let mut min_x = f32::INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    let mut count = 0usize;
    for loop_points in loops {
        for p in loop_points {
            min_x = min_x.min(p[0]);
            max_x = max_x.max(p[0]);
            min_z = min_z.min(p[2]);
            max_z = max_z.max(p[2]);
            count += 1;
        }
    }
    if count == 0 {
        log::info!("solver::ifp: {label} loops empty");
        return;
    }
    log::info!(
        "solver::ifp: {} loops={} points={} bbox=({:.3},{:.3})-({:.3},{:.3})",
        label,
        loops.len(),
        count,
        min_x,
        min_z,
        max_x,
        max_z
    );
}

fn sample_loop_points(loops: &[Vec<[f32; 3]>], spacing: f32) -> Vec<[f32; 3]> {
    let mut points = Vec::new();
    if spacing <= 0.0 {
        return points;
    }
    for loop_points in loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut acc = 0.0f32;
        for i in 0..loop_points.len() {
            let a = loop_points[i];
            let b = loop_points[(i + 1) % loop_points.len()];
            let dx = b[0] - a[0];
            let dz = b[2] - a[2];
            let len = (dx * dx + dz * dz).sqrt();
            if len == 0.0 {
                continue;
            }
            let mut t = spacing - acc;
            while t < len {
                let k = t / len;
                points.push([a[0] + dx * k, 0.0, a[2] + dz * k]);
                t += spacing;
            }
            acc = (acc + len) % spacing;
        }
    }
    points
}

fn translate_loops(loop_points: &[[f32; 3]], offsets: &[[f32; 3]]) -> Vec<Vec<[f32; 3]>> {
    let mut out = Vec::new();
    if loop_points.is_empty() {
        return out;
    }
    for offset in offsets {
        let translated = loop_points
            .iter()
            .map(|p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]])
            .collect();
        out.push(translated);
    }
    out
}
