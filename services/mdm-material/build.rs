use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // First compile common proto
    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .compile_protos(&["../../proto/common/v1/base.proto"], &["../../proto"])?;

    // Then compile material proto with extern_path for common
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("mdm_material_descriptor.bin"))
        .extern_path(".common.v1", "crate::common::v1")
        .compile_protos(&["../../proto/mdm/material/v1/material.proto"], &["../../proto"])?;

    Ok(())
}
