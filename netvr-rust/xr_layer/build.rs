use std::{env, time::SystemTime};

fn main() {
    let profile = env::var("PROFILE").unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" && profile == "debug" {
        // this is a hack to make sure that build.rs script reruns when files in
        // this crate changes

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        println!("cargo:time={}", time);
    }
}
