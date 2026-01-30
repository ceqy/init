use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let proto_files = &[
        "../../proto/sys/nr/v1/nr.proto",
        "../../proto/sys/cfg/v1/cfg.proto",
        "../../proto/common/v1/base.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("sys_core_descriptor.bin"))
        .compile_protos(proto_files, &["../../proto"])?;

    Ok(())
}
