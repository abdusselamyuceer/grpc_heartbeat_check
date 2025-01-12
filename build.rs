use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("heartbeat_descriptor.bin")).compile_protos(&["proto/heartbeat.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/heartbeat.proto")?;

    Ok(())
}