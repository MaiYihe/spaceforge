use std::env;
use std::path::Path;

#[test]
fn load_placement_region_usda_smoke() {
    if env::var("USD_CORE_TEST").ok().as_deref() != Some("1") {
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("usd_core has no parent dir");
    let path = root.join("app/assets/models/input_placement_region/chair0.usda");
    let path = path
        .to_str()
        .expect("room.usda path is not valid UTF-8");

    let region = usd_core::load_placement_region_usda(path)
        .expect("load_placement_region_usda failed");

    println!(
        "region type={} name={} count={:?} height_range={:?}",
        region.regions_type_name,
        region.name,
        region.count,
        region.height_range
    );

    if let Some(mesh) = region.restricted_region.as_ref() {
        println!(
            "restricted_region path={} points={} indices={} counts={}",
            mesh.path,
            mesh.mesh.points.len(),
            mesh.mesh.indices.len(),
            mesh.mesh.counts.len()
        );
    }

    if let Some(mesh) = region.forbidden_region.as_ref() {
        println!(
            "forbidden_region path={} points={} indices={} counts={}",
            mesh.path,
            mesh.mesh.points.len(),
            mesh.mesh.indices.len(),
            mesh.mesh.counts.len()
        );
    }

    if let Some(mesh) = region.footprint_2d.as_ref() {
        println!(
            "footprint_2d path={} points={} indices={} counts={}",
            mesh.path,
            mesh.mesh.points.len(),
            mesh.mesh.indices.len(),
            mesh.mesh.counts.len()
        );
    }

    assert!(region.restricted_region.is_some(), "missing restricted_region");
    assert!(region.forbidden_region.is_some(), "missing forbidden_region");
}

// 运行测试：USD_CORE_TEST=1 cargo test -p usd_core --test placement_region -- --nocapture
