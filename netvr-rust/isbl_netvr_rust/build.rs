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

    let maybe_copy = |filename: &str, target: Option<&str>| {
        let target = target.or(Some(filename)).unwrap();
        let result = fs::copy(
            build_path.join(filename),
            root.join("netvr-unity")
                .join("Assets")
                .join("Plugins")
                .join(target_dir.0)
                .join(target_dir.1)
                .join(target),
        );
        if result.is_ok() {
            println!(
                "cargo:rerun-if-changed={}/isbl_netvr_rust.dll",
                build_path.join(filename).to_str().unwrap()
            );
        }
        result
    };
    let copy = |filename: &str, target: Option<&str>| {
        maybe_copy(filename, target).expect("File not found");
    };

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        copy("isbl_netvr_rust.dll", Some("isbl_netvr_rust_copy.dll"));
        let _ = maybe_copy("isbl_netvr_rust.pdb", Some("isbl_netvr_rust_copy.pdb"));
        let _ = maybe_copy("isbl_netvr_rust.dll", None);
        let _ = maybe_copy("isbl_netvr_rust.pdb", None);
    } else if target_os == "android" || target_os == "linux" {
        copy("libisbl_netvr_rust.so", None);
    } else if target_os == "macos" {
        copy("libisbl_netvr_rust.dylib", None);
    } else {
        panic!("Unsupported OS {target_os}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
