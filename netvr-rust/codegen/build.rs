use std::{env, fs, path::Path};

use netvr_plugin::codegen;
use serde_reflection::{Tracer, TracerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir)
        .join("..")
        .join("..")
        .join("netvr-unity")
        .join("Packages")
        .join("cz.isbl.netvr")
        .join("Runtime");

    // Obtain the Serde format
    let mut tracer = Tracer::new(TracerConfig::default());
    tracer
        .trace_simple_type::<netvr_data::CodegenRoot>()
        .unwrap();
    let registry = tracer.registry().unwrap();

    // Create C# class
    let config = serde_generate::CodeGeneratorConfig::new("Isbl.NetVR.Binary".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode]);
    let generator = serde_generate::csharp::CodeGenerator::new(&config);

    let pkg_path = generator.write_source_files(dest_path, &registry)?;

    // write string to file
    fs::write(pkg_path.join("RPC.cs"), codegen())?;
    Ok(())
}
