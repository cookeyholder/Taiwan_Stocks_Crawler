fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::Path::new(&manifest_dir).parent().unwrap();
    let lib_dir = root.join(".cargo_lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
}
