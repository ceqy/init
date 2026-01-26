fn main() {
    // 编译 auth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/auth/api/grpc/auth_descriptor.bin")
        .out_dir("src/auth/api/grpc")
        .compile_protos(&["../../proto/iam/auth.proto"], &["../../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/user/api/grpc/user_descriptor.bin")
        .out_dir("src/user/api/grpc")
        .compile_protos(&["../../proto/iam/user.proto"], &["../../proto"])
        .expect("Failed to compile user.proto");

    // 编译 oauth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/oauth/api/grpc/oauth_descriptor.bin")
        .out_dir("src/oauth/api/grpc")
        .compile_protos(&["../../proto/iam/oauth.proto"], &["../../proto"])
        .expect("Failed to compile oauth.proto");

    println!("cargo:rerun-if-changed=../../proto/iam/auth.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/user.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/oauth.proto");
}
