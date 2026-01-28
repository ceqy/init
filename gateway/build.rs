fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    
    // 编译 auth.proto
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(&["../proto/iam/v1/auth.proto"], &["../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(&["../proto/iam/v1/user.proto"], &["../proto"])
        .expect("Failed to compile user.proto");

    // 编译 audit.proto
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(&["../proto/iam/v1/audit.proto"], &["../proto"])
        .expect("Failed to compile audit.proto");

    println!("cargo:rerun-if-changed=../proto/iam/v1/auth.proto");
    println!("cargo:rerun-if-changed=../proto/iam/v1/user.proto");
    println!("cargo:rerun-if-changed=../proto/iam/v1/audit.proto");
}
