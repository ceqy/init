//! Proto 转换模块
//!
//! 提供统一的转换函数，将领域模型转换为 Proto 消息

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

/// 将 DateTime 转换为 Timestamp
pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

/// 将可选的字符串转换为默认值
pub fn option_string_to_default(opt: Option<String>) -> String {
    opt.unwrap_or_default()
}

// ============ 以下是各个服务的转换函数 ============

// RBAC 服务转换函数
// role_to_proto 和 permission_to_proto 仍在各自的 service 文件中
// 因为它们依赖具体的 Proto 类型

// Policy 服务转换函数
// policy_to_proto 仍在 policy_service.rs 中
// 因为它依赖具体的 Proto 类型

// 如果需要进一步重构，可以考虑使用泛型或宏来统一这些转换函数
