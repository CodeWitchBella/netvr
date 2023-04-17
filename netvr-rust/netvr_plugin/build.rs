use std::{env, fs, path::Path, time::SystemTime};

fn main() {
    let build_path = Path::new(&env::var("OUT_DIR").unwrap())
        .join("..")
        .join("..")
        .join("..");
    let profile = env::var("PROFILE").unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" && profile == "debug" {
        // windows' MSVC hard-codes path to debug info (.pdb) files upon linking
        // it becomes locked when rust tries to produce stack trace for log/error/panic
        // there is also no simple way to unlock the PDB file when unloading related dll
        //
        // ...this combined means that if I want debug info without having to
        // restart unity every time I use said info, I have to choose unique pdb
        // filename for each compilation.
        //
        // The way I generate the name might break in the event of time change
        // (eg. after NTP sync), but if it breaks I either fail compilation once
        // (in case of repeat value), or more likely loose debug info (in case
        // of time going backwards). Neither of those is a big problem and therefore
        // I choose to ignore this.

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
