fn main() {
    // When the audio feature is enabled, we need to ensure the opus library is linked.
    // opus-static-sys builds opus and sets up link paths, but we need to propagate them.
    #[cfg(feature = "audio")]
    {
        // Find the opus-static-sys build output directory
        // The OUT_DIR pattern for dependencies is: target/{profile}/build/{crate-name}-{hash}/out
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            // out_dir is something like: target/debug/build/crabcamera-xxx/out
            // We need to find: target/debug/build/opus-static-sys-xxx/out/lib
            let target_dir = std::path::Path::new(&out_dir)
                .parent() // build/crabcamera-xxx
                .and_then(|p| p.parent()) // build
                .expect("Could not find build directory");
            
            // Search for opus-static-sys output directory
            if let Ok(entries) = std::fs::read_dir(target_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("opus-static-sys-") {
                        let opus_lib_dir = entry.path().join("out").join("lib");
                        if opus_lib_dir.exists() {
                            println!("cargo:rustc-link-search=native={}", opus_lib_dir.display());
                            println!("cargo:rustc-link-lib=static=opus");
                            println!("cargo:rerun-if-changed={}", opus_lib_dir.display());
                            return;
                        }
                    }
                }
            }
        }
        
        // Fallback: try DEP_ variable (works for some build configurations)
        if let Ok(lib_path) = std::env::var("DEP_OPUS_LIB_DIR") {
            println!("cargo:rustc-link-search=native={}", lib_path);
            println!("cargo:rustc-link-lib=static=opus");
        }
    }
}
