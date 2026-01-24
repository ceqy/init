//! 通用工具函数

use uuid::Uuid;

/// 生成新的 UUID v7（时间有序）
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

/// 生成新的 UUID v4（随机）
pub fn random_id() -> Uuid {
    Uuid::new_v4()
}
