fn main() {
    println!("cargo:rerun-if-changed=cpp/vdb_wrapper.cpp");

    cc::Build::new()
        .cpp(true)
        .file("cpp/vdb_wrapper.cpp")
        .include("/usr/include")
        .flag("-std=c++17")
        .compile("vdb_wrapper");

    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    println!("cargo:rustc-link-lib=openvdb");
    println!("cargo:rustc-link-lib=tbb");
    println!("cargo:rustc-link-lib=blosc");
    println!("cargo:rustc-link-lib=Imath");

    // ⭐ Boost（关键）
    println!("cargo:rustc-link-lib=boost_system");
    println!("cargo:rustc-link-lib=boost_thread");
    println!("cargo:rustc-link-lib=boost_filesystem");
}
