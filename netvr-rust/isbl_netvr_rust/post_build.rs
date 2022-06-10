use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let build_path_string = env::var("CRATE_OUT_DIR").unwrap();

    let build_path = Path::new(&build_path_string);
    let pdb_dir = build_path.join("pdb");
    // cleanup old pdbs
    if let Ok(file_interator) = fs::read_dir(pdb_dir.clone()) {
        let mut pdbs: Vec<PathBuf> = file_interator
            .filter_map(|entry| match entry {
                Ok(v) => {
                    let path_buf = v.path();
                    let path = path_buf.as_path();
                    if let Some(Some(filename)) = path.file_name().map(|p| p.to_str()) {
                        if filename.starts_with("isbl_netvr_rust") {
                            Some(path_buf)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect();
        pdbs.sort_unstable();
        let _last = pdbs.remove(pdbs.len() - 1);
        for path in pdbs {
            let _ = fs::remove_file(path.clone());
            println!("Name: {}", path.display());
        }
    }

    let root = Path::new(&env::var("CRATE_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..");
    let copy = |filename: &str, dir1: &str, dir2: &str| {
        let _ = fs::copy(
            build_path.join(filename),
            root.join("netvr-unity")
                .join("Assets")
                .join("Plugins")
                .join(dir1)
                .join(dir2)
                .join(filename),
        );
    };

    let profile = env::var("CRATE_PROFILE").unwrap();
    if profile == "release" {
        println!("Copying dynamic libraries to unity project");
        copy("isbl_netvr_rust.dll", "Windows", "x64");
        copy("libisbl_netvr_rust.so", "Android", "arm64-v8a");
    }
}
