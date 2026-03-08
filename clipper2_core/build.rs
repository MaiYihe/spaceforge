use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/clipper2_wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/clipper2_stub.cpp");
    let force_stub = std::env::var("CLIPPER2_STUB").ok().as_deref() == Some("1")
        || std::env::var("CARGO_FEATURE_STUB").is_ok();

    let lib_root = std::env::var("CLIPPER2_ROOT").ok().map(PathBuf::from);
    let vendored = PathBuf::from("third_party/clipper2/CPP");
    let has_vendored = vendored.join("Clipper2Lib/include").exists();

    let mut build = cc::Build::new();
    build.cpp(true).flag_if_supported("-std=c++17");

    if !force_stub {
        if let Some(root) = lib_root {
            println!("cargo:warning=clipper2_core: using CLIPPER2_ROOT sources");
            let include = root.join("Clipper2Lib/include");
            let src = root.join("Clipper2Lib/src");
            build.include(include);
            if src.exists() {
                build.file(src.join("clipper.engine.cpp"));
                build.file(src.join("clipper.offset.cpp"));
                build.file(src.join("clipper.rectclip.cpp"));
                build.file(src.join("clipper.triangulation.cpp"));
            }
            build.define("CLIPPER2_AVAILABLE", None);
            build.file("cpp/clipper2_wrapper.cpp");
            build.compile("clipper2_core");
            println!("cargo:warning=clipper2_core: rebuilt from CLIPPER2_ROOT");
            println!("cargo:rerun-if-env-changed=CLIPPER2_ROOT");
            return;
        }
        if has_vendored {
            println!("cargo:warning=clipper2_core: using vendored Clipper2");
            build.include(vendored.join("Clipper2Lib/include"));
            let src = vendored.join("Clipper2Lib/src");
            build.file(src.join("clipper.engine.cpp"));
            build.file(src.join("clipper.offset.cpp"));
            build.file(src.join("clipper.rectclip.cpp"));
            build.file(src.join("clipper.triangulation.cpp"));
            build.define("CLIPPER2_AVAILABLE", None);
            build.file("cpp/clipper2_wrapper.cpp");
            build.compile("clipper2_core");
            println!("cargo:warning=clipper2_core: rebuilt from vendored sources");
            return;
        }
    }

    build.file("cpp/clipper2_stub.cpp");
    build.compile("clipper2_core");
    println!("cargo:warning=clipper2_core: rebuilt stub");
    println!("cargo:warning=Clipper2 headers not found; building stub (set CLIPPER2_ROOT or vendor to third_party/clipper2)");
}
