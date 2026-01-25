fn main() {
    // 编译 auth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/auth/api/grpc")
        .compile_protos(&["../../proto/iam/auth.proto"], &["../../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/user/api/grpc")
        .compile_protos(&["../../proto/iam/user.proto"], &["../../proto"])
        .expect("Failed to compile user.proto");

    println!("cargo:rerun-if-changed=../../proto/iam/auth.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/user.proto");
}
