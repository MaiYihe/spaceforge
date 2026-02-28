use std::env;

#[test]
fn load_space_usda_smoke() {
    if env::var("USD_CORE_TEST").ok().as_deref() != Some("1") {
        return;
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("usd_core has no parent dir");
    let path = root.join("app/assets/models/input_space/room.usda");
    let path = path
        .to_str()
        .expect("room.usda path is not valid UTF-8");
    let meshes = usd_core::load_space_usda(path).expect("load_space_usda failed");
    println!("loaded {} meshes", meshes.len());
    for (idx, mesh) in meshes.iter().enumerate() {
        println!(
            "[{idx}] path={} points={} indices={} counts={} mask={:?}",
            mesh.mesh.path,
            mesh.mesh.mesh.points.len(),
            mesh.mesh.mesh.indices.len(),
            mesh.mesh.mesh.counts.len(),
            mesh.regions_type_mask
        );
    }
    assert!(!meshes.is_empty(), "expected at least one mesh");
}

// 运行测试：USD_CORE_TEST=1 cargo test -p usd_core --test space -- --nocapture
