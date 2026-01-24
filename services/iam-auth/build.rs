fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/api/grpc")
        .compile_protos(
            &["../../proto/iam/auth.proto"],
            &["../../proto"],
        )
        .expect("Failed to compile protos");

    println!("cargo:rerun-if-changed=../../proto/iam/auth.proto");
}
