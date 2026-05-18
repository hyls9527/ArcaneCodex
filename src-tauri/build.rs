/// MSYS2 creates symlinks with Unix paths that Windows PE loader can't resolve.
/// Replace DirectML.dll symlinks in the target directory with real file copies.
fn fix_onnx_symlinks() {
    // OUT_DIR is like target/debug/build/xxx/out — go up 3 to get target/debug
    let out_dir = std::path::PathBuf::from(
        std::env::var("OUT_DIR").unwrap_or_default(),
    );
    let target_profile = out_dir
        .parent() // build/xxx
        .and_then(|p| p.parent()) // build
        .and_then(|p| p.parent()); // target/debug
    if let Some(deps) = target_profile.map(|p| p.join("deps")) {
        let dll = deps.join("DirectML.dll");
        if dll.exists() && dll.is_symlink() {
            if let Ok(target) = std::fs::read_link(&dll) {
                if let Ok(data) = std::fs::read(&target) {
                    let _ = std::fs::remove_file(&dll);
                    let _ = std::fs::write(&dll, &data);
                }
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    tauri_build::build();
    fix_onnx_symlinks();
}
