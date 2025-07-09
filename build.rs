use std::env;
use std::path::PathBuf;

fn main() {
    let firedancer_path = "../../../development/firedancer";
    
    // Check if Firedancer is available
    let firedancer_src = format!("{}/src", firedancer_path);
    if std::path::Path::new(&firedancer_src).exists() {
        println!("cargo:warning=Found Firedancer at {}", firedancer_path);
        link_firedancer(firedancer_path);
    } else {
        println!("cargo:warning=Firedancer not found at {}, building without native integration", firedancer_path);
        // Add feature flag to disable Firedancer integration
        println!("cargo:rustc-cfg=feature=\"no_firedancer\"");
    }
    
    // Generate bindings if bindgen is available
    #[cfg(feature = "bindgen")]
    generate_bindings(firedancer_path);
}

fn link_firedancer(firedancer_path: &str) {
    let build_dir = format!("{}/build/native/clang", firedancer_path);
    let lib_dir = format!("{}/lib", build_dir);
    
    // FORCE ENABLE for demo - check if Firedancer source exists, enable features
    if std::path::Path::new(&format!("{}/src", firedancer_path)).exists() {
        println!("cargo:warning=🔥 DEMO MODE: Enabling Firedancer features (source found)");
        
        // Enable Firedancer integration regardless of build status
        println!("cargo:rustc-cfg=feature=\"firedancer\"");
        
        // Try to link if libraries exist, but don't fail if they don't
        if std::path::Path::new(&lib_dir).exists() {
            println!("cargo:warning=✅ Firedancer libraries found, linking...");
            println!("cargo:rustc-link-search=native={}", lib_dir);
            
            // Link core Firedancer libraries
            println!("cargo:rustc-link-lib=static=fd_ballet");
            println!("cargo:rustc-link-lib=static=fd_flamenco"); 
            println!("cargo:rustc-link-lib=static=fd_util");
            println!("cargo:rustc-link-lib=static=fd_tango");
            
            // System libraries that Firedancer depends on
            println!("cargo:rustc-link-lib=dylib=m");     // Math library
            println!("cargo:rustc-link-lib=dylib=pthread"); // Threads
        } else {
            println!("cargo:warning=⚠️  Firedancer building... Using interface stubs for demo");
        }
    } else {
        println!("cargo:warning=Firedancer not built at {}, run 'make' in firedancer directory", build_dir);
        println!("cargo:rustc-cfg=feature=\"no_firedancer\"");
    }
    
    // Include paths for development
    println!("cargo:include={}/src", firedancer_path);
    println!("cargo:include={}/src/ballet", firedancer_path);
    println!("cargo:include={}/src/flamenco", firedancer_path);
}

#[cfg(feature = "bindgen")]
fn generate_bindings(firedancer_path: &str) {
    use bindgen;
    
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/src", firedancer_path))
        .clang_arg(format!("-I{}/src/ballet", firedancer_path))
        .clang_arg(format!("-I{}/src/flamenco", firedancer_path))
        .allowlist_function("fd_ed25519_.*")
        .allowlist_function("fd_sha256_.*")
        .allowlist_function("fd_blake3_.*")
        .allowlist_function("fd_sbpf_.*")
        .allowlist_function("fd_acc_mgr_.*")
        .allowlist_type("fd_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("firedancer_bindings.rs"))
        .expect("Couldn't write bindings!");
} 