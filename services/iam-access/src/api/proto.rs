//! gRPC 生成代码引入

pub mod rbac {
    tonic::include_proto!("cuba.iam.rbac");
}

pub mod policy {
    tonic::include_proto!("cuba.iam.policy");
}

pub mod authorization {
    tonic::include_proto!("cuba.iam.authorization");
}
