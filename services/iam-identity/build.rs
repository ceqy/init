fn main() {
    // 创建统一的输出目录
    std::fs::create_dir_all("src/api/grpc/proto_gen").ok();

    // 编译 auth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/api/grpc/proto_gen/auth_descriptor.bin")
        .out_dir("src/api/grpc/proto_gen")
        .compile_protos(&["../../proto/iam/v1/auth.proto"], &["../../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/api/grpc/proto_gen/user_descriptor.bin")
        .out_dir("src/api/grpc/proto_gen")
        .compile_protos(&["../../proto/iam/v1/user.proto"], &["../../proto"])
        .expect("Failed to compile user.proto");

    // 编译 oauth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/api/grpc/proto_gen/oauth_descriptor.bin")
        .out_dir("src/api/grpc/proto_gen")
        .compile_protos(&["../../proto/iam/v1/oauth.proto"], &["../../proto"])
        .expect("Failed to compile oauth.proto");

    // 编译 audit.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/api/grpc/proto_gen/audit_descriptor.bin")
        .out_dir("src/api/grpc/proto_gen")
        .compile_protos(&["../../proto/iam/v1/audit.proto"], &["../../proto"])
        .expect("Failed to compile audit.proto");

    println!("cargo:rerun-if-changed=../../proto/iam/v1/auth.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/v1/user.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/v1/oauth.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/v1/audit.proto");
}
