//! gRPC 生成代码引入

pub mod rbac {
    tonic::include_proto!("cuba.iam.rbac.v1");
}

pub mod policy {
    tonic::include_proto!("cuba.iam.policy.v1");
}

pub mod authorization {
    tonic::include_proto!("cuba.iam.authorization.v1");
}
