use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let proto_files = &[
        "../../proto/sc/df/v1/df.proto",
        "../../proto/common/v1/base.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("sc_df_descriptor.bin"))
        .compile_protos(proto_files, &["../../proto"])?;

    Ok(())
}
