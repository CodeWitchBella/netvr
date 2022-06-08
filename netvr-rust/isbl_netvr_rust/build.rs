use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..");

    let build_path = Path::new(&env::var("OUT_DIR").unwrap())
        .join("..")
        .join("..")
        .join("..");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let target_dir = (if target_arch == "x86_64" {
        Ok(("Windows", "x64"))
    } else if target_arch == "aarch64" {
        Ok(("Android", "arm64-v8a"))
    } else {
        Err("Unsuported arch")
    })
    .unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let filename = if target_os == "windows" {
        Ok("isbl_netvr_rust.dll")
    } else if target_os == "android" || target_os == "linux" {
        Ok("libisbl_netvr_rust.so")
    } else if target_os == "macos" {
        Ok("libisbl_netvr_rust.dylib")
    } else {
        Err(format!("Unsupported OS {target_os}"))
    }
    .unwrap();

    fs::copy(
        build_path.join(filename),
        root.join("netvr-unity")
            .join("Assets")
            .join("Plugins")
            .join(target_dir.0)
            .join(target_dir.1)
            .join(filename),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}/isbl_netvr_rust.dll",
        build_path.join(filename).to_str().unwrap()
    );
}
