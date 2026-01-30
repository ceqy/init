//! gRPC 生成代码引入

pub mod rbac {
    tonic::include_proto!("iam.rbac.v1");
}

pub mod policy {
    tonic::include_proto!("iam.policy.v1");
}

pub mod authorization {
    tonic::include_proto!("iam.authorization.v1");
}
