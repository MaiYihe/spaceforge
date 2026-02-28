use std::env;
use std::path::Path;
use serde_json::json;

#[test]
fn load_space_model_smoke() {
    if env::var("USD_CORE_TEST").ok().as_deref() != Some("1") {
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("asset_import has no parent dir");
    let path = root.join("app/assets/models/input_space/room.usda");
    let path = path
        .to_str()
        .expect("room.usda path is not valid UTF-8");

    let regions_type_path = root.join("app/config/regions_types.toml");
    let regions_type_path = regions_type_path
        .to_str()
        .expect("regions_type.toml path is not valid UTF-8");
    let regions_type_ids = asset_import::load_regions_type_registry(regions_type_path)
        .expect("load_regions_type_registry failed");

    let space = asset_import::load_space_model_from_usda(path, &regions_type_ids, 1.0)
        .expect("load_space_model_from_usda failed");

    let mut meshes_out = Vec::new();
    for (idx, mesh) in space.meshes.iter().enumerate() {
        let vertices = mesh.positions.len();
        let meta = space.surface_metas.get(idx);
        let mask_bits = meta.map(|m| m.regions_type_mask.bits());
        let mask_binary = mask_bits.map(|bits| format!("{:032b}", bits));
        meshes_out.push(json!({
            "mesh_id": idx,
            "mesh": { "vertices": vertices },
            "regions_type_mask_binary": mask_binary
        }));
    }

    let payload = json!({
        "meshes": meshes_out
    });

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    assert!(!space.meshes.is_empty(), "expected at least one mesh");
    assert_eq!(space.meshes.len(), space.surface_metas.len());
}

//  USD_CORE_TEST=1 cargo test -p asset_import --test space -- --nocapture
