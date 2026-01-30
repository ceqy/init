use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let proto_files = &[
        "../../proto/fi/co/v1/co.proto",
        "../../proto/common/v1/base.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("fi_co_descriptor.bin"))
        .compile_protos(proto_files, &["../../proto"])?;

    Ok(())
}
