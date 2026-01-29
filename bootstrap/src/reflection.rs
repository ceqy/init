//! gRPC 反射辅助工具
//!
//! 提供统一的 gRPC 反射服务构建方式。

pub use tonic_reflection::server::Builder;
pub use tonic_reflection::server::v1::{ServerReflection, ServerReflectionServer};

/// 创建一个新的反射服务构建器
pub fn reflection_builder() -> Builder<'static> {
    Builder::configure()
}

/// 构建一个包含指定文件描述符集的反射服务
pub fn build_reflection(
    file_descriptor_sets: Vec<&'static [u8]>,
) -> Result<ServerReflectionServer<impl ServerReflection>, Box<dyn std::error::Error>> {
    let mut builder = Builder::configure();
    for fds in file_descriptor_sets {
        builder = builder.register_encoded_file_descriptor_set(fds);
    }
    Ok(builder.build_v1()?)
}
