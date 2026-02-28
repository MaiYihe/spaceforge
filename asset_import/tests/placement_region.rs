use std::env;
use std::path::Path;
use serde_json::json;

#[test]
fn load_placement_region_model_smoke() {
    if env::var("USD_CORE_TEST").ok().as_deref() != Some("1") {
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("asset_import has no parent dir");
    let path = root.join("app/assets/models/input_placement_region/chair0.usda");
    let path = path
        .to_str()
        .expect("chair0.usda path is not valid UTF-8");

    let regions_type_path = root.join("app/config/regions_types.toml");
    let regions_type_path = regions_type_path
        .to_str()
        .expect("regions_type.toml path is not valid UTF-8");
    let regions_type_ids = asset_import::load_regions_type_registry(regions_type_path)
        .expect("load_regions_type_registry failed");

    let region = asset_import::load_placement_region_model_from_usda(path, &regions_type_ids, 1.0)
        .expect("load_placement_region_model_from_usda failed");

    let restricted_vertices = region.regions.restricted_region.mesh.positions.len();
    let forbidden_vertices = region.regions.forbidden_region.mesh.positions.len();
    let footprint_vertices = region.visual.footprint_2d.positions.len();

    let payload = json!({
        "regions": {
            "restricted_region": { "mesh": { "vertices": restricted_vertices } },
            "forbidden_region": { "mesh": { "vertices": forbidden_vertices } }
        },
        "semantics": {
            "regions_type": region.semantics.regions_type,
            "count": region.semantics.count
        },
        "visual": {
            "footprint_2d": { "mesh": { "vertices": footprint_vertices } },
            "height_range": {
                "min_y": region.visual.height_range.min_y,
                "max_y": region.visual.height_range.max_y
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    assert!(
        !region.regions.restricted_region.mesh.positions.is_empty(),
        "restricted_region should have positions"
    );
    assert!(
        !region.regions.forbidden_region.mesh.positions.is_empty(),
        "forbidden_region should have positions"
    );
}

// USD_CORE_TEST=1 cargo test -p asset_import --test placement_region -- --nocapture
