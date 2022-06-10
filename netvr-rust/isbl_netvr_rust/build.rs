use std::path::Path;
use std::time::SystemTime;
use std::{env, fs};

fn main() {
    let build_path = Path::new(&env::var("OUT_DIR").unwrap())
        .join("..")
        .join("..")
        .join("..");
    let profile = env::var("PROFILE").unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" && profile == "debug" {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let _ = fs::create_dir(build_path.join("pdb"));
        let pdb = build_path
            .join("pdb")
            .join(format!("isbl_netvr_rust-{time}.pdb"));
        print!("cargo:rustc-link-arg=/pdb:");
        println!("{}", pdb.display());
    }
}
