//! 布隆过滤器实现
//!
//! 用于防止缓存穿透：
//! - 在查询缓存前，先检查布隆过滤器
//! - 如果布隆过滤器判断 key 不存在，直接返回，避免查询数据库
//! - 适用于大量不存在的 key 的场景

use cuba_errors::{AppError, AppResult};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};

/// 基于 Redis 的布隆过滤器
pub struct RedisBloomFilter {
    conn: ConnectionManager,
    /// 过滤器名称
    name: String,
    /// 预期元素数量
    expected_items: u64,
    /// 期望的误判率
    false_positive_rate: f64,
}

impl RedisBloomFilter {
    pub fn new(
        conn: ConnectionManager,
        name: String,
        expected_items: u64,
        false_positive_rate: f64,
    ) -> Self {
        Self {
            conn,
            name,
            expected_items,
            false_positive_rate,
        }
    }

    /// 初始化布隆过滤器
    /// 使用 Redis BF.RESERVE 命令（需要 RedisBloom 模块）
    pub async fn reserve(&self) -> AppResult<()> {
        let mut conn = self.conn.clone();

        // BF.RESERVE {key} {error_rate} {capacity}
        let result: Result<String, redis::RedisError> = redis::cmd("BF.RESERVE")
            .arg(&self.name)
            .arg(self.false_positive_rate)
            .arg(self.expected_items)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // 如果已存在，忽略错误
                if e.to_string().contains("item exists") {
                    Ok(())
                } else {
                    Err(AppError::internal(format!(
                        "Failed to reserve bloom filter: {}",
                        e
                    )))
                }
            }
        }
    }

    /// 添加元素到布隆过滤器
    pub async fn add(&self, item: &str) -> AppResult<bool> {
        let mut conn = self.conn.clone();

        // BF.ADD {key} {item}
        redis::cmd("BF.ADD")
            .arg(&self.name)
            .arg(item)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to add to bloom filter: {}", e)))
    }

    /// 批量添加元素
    pub async fn add_multi(&self, items: &[&str]) -> AppResult<Vec<bool>> {
        let mut conn = self.conn.clone();

        // BF.MADD {key} {item1} {item2} ...
        let mut cmd = redis::cmd("BF.MADD");
        cmd.arg(&self.name);
        for item in items {
            cmd.arg(item);
        }

        cmd.query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to add multi to bloom filter: {}", e)))
    }

    /// 检查元素是否可能存在
    /// 返回 true 表示可能存在（有误判率）
    /// 返回 false 表示一定不存在
    pub async fn exists(&self, item: &str) -> AppResult<bool> {
        let mut conn = self.conn.clone();

        // BF.EXISTS {key} {item}
        redis::cmd("BF.EXISTS")
            .arg(&self.name)
            .arg(item)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to check bloom filter: {}", e)))
    }

    /// 批量检查元素
    pub async fn exists_multi(&self, items: &[&str]) -> AppResult<Vec<bool>> {
        let mut conn = self.conn.clone();

        // BF.MEXISTS {key} {item1} {item2} ...
        let mut cmd = redis::cmd("BF.MEXISTS");
        cmd.arg(&self.name);
        for item in items {
            cmd.arg(item);
        }

        cmd.query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to check multi bloom filter: {}", e)))
    }

    /// 获取布隆过滤器信息
    pub async fn info(&self) -> AppResult<BloomFilterInfo> {
        let mut conn = self.conn.clone();

        // BF.INFO {key}
        let info: Vec<redis::Value> = redis::cmd("BF.INFO")
            .arg(&self.name)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to get bloom filter info: {}", e)))?;

        // 解析返回的信息
        let mut capacity = 0u64;
        let mut size = 0u64;
        let mut num_filters = 0u64;
        let mut num_items = 0u64;

        let mut i = 0;
        while i < info.len() {
            if let redis::Value::BulkString(ref key) = info[i] {
                let key_str = String::from_utf8_lossy(key);
                if i + 1 < info.len() {
                    match key_str.as_ref() {
                        "Capacity" => {
                            if let redis::Value::Int(val) = info[i + 1] {
                                capacity = val as u64;
                            }
                        }
                        "Size" => {
                            if let redis::Value::Int(val) = info[i + 1] {
                                size = val as u64;
                            }
                        }
                        "Number of filters" => {
                            if let redis::Value::Int(val) = info[i + 1] {
                                num_filters = val as u64;
                            }
                        }
                        "Number of items inserted" => {
                            if let redis::Value::Int(val) = info[i + 1] {
                                num_items = val as u64;
                            }
                        }
                        _ => {}
                    }
                }
            }
            i += 2;
        }

        Ok(BloomFilterInfo {
            capacity,
            size,
            num_filters,
            num_items,
        })
    }
}

/// 布隆过滤器信息
#[derive(Debug, Clone)]
pub struct BloomFilterInfo {
    /// 容量
    pub capacity: u64,
    /// 大小（字节）
    pub size: u64,
    /// 过滤器数量
    pub num_filters: u64,
    /// 已插入元素数量
    pub num_items: u64,
}

/// 简单的内存布隆过滤器（不依赖 Redis 模块）
/// 使用多个 hash 函数和 bitmap 实现
pub struct SimpleBloomFilter {
    conn: ConnectionManager,
    /// 过滤器名称
    name: String,
    /// bitmap 大小（位）
    size: u64,
    /// hash 函数数量
    num_hashes: u32,
}

impl SimpleBloomFilter {
    /// 创建简单布隆过滤器
    /// 根据预期元素数量和误判率计算最优参数
    pub fn new(
        conn: ConnectionManager,
        name: String,
        expected_items: u64,
        false_positive_rate: f64,
    ) -> Self {
        // 计算最优 bitmap 大小
        let size = Self::optimal_size(expected_items, false_positive_rate);
        // 计算最优 hash 函数数量
        let num_hashes = Self::optimal_num_hashes(size, expected_items);

        Self {
            conn,
            name,
            size,
            num_hashes,
        }
    }

    /// 计算最优 bitmap 大小
    /// m = -n * ln(p) / (ln(2)^2)
    fn optimal_size(n: u64, p: f64) -> u64 {
        let m = -(n as f64) * p.ln() / (2.0_f64.ln().powi(2));
        m.ceil() as u64
    }

    /// 计算最优 hash 函数数量
    /// k = (m / n) * ln(2)
    fn optimal_num_hashes(m: u64, n: u64) -> u32 {
        let k = (m as f64 / n as f64) * 2.0_f64.ln();
        k.ceil().max(1.0) as u32
    }

    /// 计算 hash 值
    fn hash(&self, item: &str, seed: u32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() % self.size
    }

    /// 添加元素
    pub async fn add(&self, item: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();

        // 使用 Lua 脚本批量设置多个 bit
        let script = Script::new(
            r"
            for i = 1, #ARGV do
                redis.call('SETBIT', KEYS[1], ARGV[i], 1)
            end
            return 1
            ",
        );

        let positions: Vec<u64> = (0..self.num_hashes).map(|i| self.hash(item, i)).collect();

        let _: i64 = script
            .key(&self.name)
            .arg(positions)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to add to bloom filter: {}", e)))?;

        Ok(())
    }

    /// 检查元素是否可能存在
    pub async fn exists(&self, item: &str) -> AppResult<bool> {
        let mut conn = self.conn.clone();

        // 使用 Lua 脚本批量检查多个 bit
        let script = Script::new(
            r"
            for i = 1, #ARGV do
                if redis.call('GETBIT', KEYS[1], ARGV[i]) == 0 then
                    return 0
                end
            end
            return 1
            ",
        );

        let positions: Vec<u64> = (0..self.num_hashes).map(|i| self.hash(item, i)).collect();

        let exists: i64 = script
            .key(&self.name)
            .arg(positions)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to check bloom filter: {}", e)))?;

        Ok(exists == 1)
    }

    /// 清空布隆过滤器
    pub async fn clear(&self) -> AppResult<()> {
        let mut conn = self.conn.clone();
        conn.del(&self.name)
            .await
            .map_err(|e| AppError::internal(format!("Failed to clear bloom filter: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_size_calculation() {
        // 1000 个元素，1% 误判率
        let size = SimpleBloomFilter::optimal_size(1000, 0.01);
        assert!(size > 9000 && size < 10000); // 约 9585
    }

    #[test]
    fn test_optimal_num_hashes_calculation() {
        let size = 9585;
        let n = 1000;
        let num_hashes = SimpleBloomFilter::optimal_num_hashes(size, n);
        assert_eq!(num_hashes, 7); // 约 6.64，向上取整为 7
    }
}
