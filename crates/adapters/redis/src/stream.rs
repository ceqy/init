//! Redis Stream 模块
//!
//! 提供 Redis Stream 消息队列功能

use std::collections::HashMap;

use errors::{AppError, AppResult};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisResult, Value};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::config::RedisConfig;

/// Stream 消息
#[derive(Debug, Clone)]
pub struct StreamMessage {
    /// 消息 ID
    pub id: String,
    /// Stream 名称
    pub stream: String,
    /// 消息字段
    pub fields: HashMap<String, String>,
}

impl StreamMessage {
    /// 获取字段值
    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }

    /// 获取 JSON 字段并解析
    pub fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> AppResult<Option<T>> {
        match self.fields.get(key) {
            Some(value) => {
                let parsed = serde_json::from_str(value)
                    .map_err(|e| AppError::validation(format!("Failed to parse JSON: {}", e)))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }
}

/// Stream 生产者
pub struct StreamProducer {
    conn: ConnectionManager,
    key_prefix: Option<String>,
    max_len: Option<usize>,
}

impl StreamProducer {
    /// 创建新的生产者
    pub async fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::internal(format!("Failed to create connection: {}", e)))?;

        Ok(Self {
            conn,
            key_prefix: config.key_prefix.clone(),
            max_len: None,
        })
    }

    /// 从连接管理器创建
    pub fn from_connection(conn: ConnectionManager) -> Self {
        Self {
            conn,
            key_prefix: None,
            max_len: None,
        }
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 设置最大长度（自动裁剪）
    pub fn with_max_len(mut self, max_len: usize) -> Self {
        self.max_len = Some(max_len);
        self
    }

    /// 获取带前缀的 Stream 名
    fn prefixed_stream(&self, stream: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, stream),
            None => stream.to_string(),
        }
    }

    /// 添加消息到 Stream
    pub async fn xadd(
        &mut self,
        stream: &str,
        fields: &[(&str, &str)],
    ) -> AppResult<String> {
        let stream = self.prefixed_stream(stream);

        let mut cmd = redis::cmd("XADD");
        cmd.arg(&stream);

        // 可选的 MAXLEN
        if let Some(max_len) = self.max_len {
            cmd.arg("MAXLEN").arg("~").arg(max_len);
        }

        cmd.arg("*"); // 自动生成 ID

        for (key, value) in fields {
            cmd.arg(*key).arg(*value);
        }

        let id: String = cmd
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XADD: {}", e)))?;

        debug!(stream = %stream, id = %id, "Message added to stream");
        Ok(id)
    }

    /// 添加 JSON 消息
    pub async fn xadd_json<T: Serialize>(
        &mut self,
        stream: &str,
        key: &str,
        data: &T,
    ) -> AppResult<String> {
        let payload = serde_json::to_string(data)
            .map_err(|e| AppError::internal(format!("Failed to serialize: {}", e)))?;
        self.xadd(stream, &[(key, &payload)]).await
    }

    /// 获取 Stream 长度
    pub async fn xlen(&mut self, stream: &str) -> AppResult<usize> {
        let stream = self.prefixed_stream(stream);
        let len: usize = self
            .conn
            .xlen(&stream)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XLEN: {}", e)))?;
        Ok(len)
    }

    /// 裁剪 Stream
    pub async fn xtrim(&mut self, stream: &str, max_len: usize) -> AppResult<usize> {
        let stream = self.prefixed_stream(stream);
        let trimmed: usize = redis::cmd("XTRIM")
            .arg(&stream)
            .arg("MAXLEN")
            .arg("~")
            .arg(max_len)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XTRIM: {}", e)))?;
        Ok(trimmed)
    }
}

/// Stream 消费者
pub struct StreamConsumer {
    conn: ConnectionManager,
    key_prefix: Option<String>,
    group: String,
    consumer: String,
    block_ms: Option<usize>,
}

impl StreamConsumer {
    /// 创建新的消费者
    pub async fn new(
        config: &RedisConfig,
        group: impl Into<String>,
        consumer: impl Into<String>,
    ) -> AppResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::internal(format!("Failed to create connection: {}", e)))?;

        Ok(Self {
            conn,
            key_prefix: config.key_prefix.clone(),
            group: group.into(),
            consumer: consumer.into(),
            block_ms: Some(5000),
        })
    }

    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 设置阻塞超时（毫秒）
    pub fn with_block_ms(mut self, ms: usize) -> Self {
        self.block_ms = Some(ms);
        self
    }

    /// 禁用阻塞
    pub fn without_block(mut self) -> Self {
        self.block_ms = None;
        self
    }

    /// 获取带前缀的 Stream 名
    fn prefixed_stream(&self, stream: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, stream),
            None => stream.to_string(),
        }
    }

    /// 创建消费者组（如果不存在）
    pub async fn create_group(&mut self, stream: &str, start_id: &str) -> AppResult<()> {
        let stream = self.prefixed_stream(stream);

        let result: RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&stream)
            .arg(&self.group)
            .arg(start_id)
            .arg("MKSTREAM")
            .query_async(&mut self.conn)
            .await;

        match result {
            Ok(()) => {
                info!(stream = %stream, group = %self.group, "Consumer group created");
                Ok(())
            }
            Err(e) => {
                // 忽略 "BUSYGROUP" 错误（组已存在）
                if e.to_string().contains("BUSYGROUP") {
                    debug!(stream = %stream, group = %self.group, "Consumer group already exists");
                    Ok(())
                } else {
                    Err(AppError::internal(format!("Failed to create group: {}", e)))
                }
            }
        }
    }

    /// 读取消息
    pub async fn xreadgroup(
        &mut self,
        streams: &[&str],
        count: usize,
    ) -> AppResult<Vec<StreamMessage>> {
        let prefixed_streams: Vec<String> = streams
            .iter()
            .map(|s| self.prefixed_stream(s))
            .collect();

        let mut cmd = redis::cmd("XREADGROUP");
        cmd.arg("GROUP")
            .arg(&self.group)
            .arg(&self.consumer)
            .arg("COUNT")
            .arg(count);

        if let Some(block_ms) = self.block_ms {
            cmd.arg("BLOCK").arg(block_ms);
        }

        cmd.arg("STREAMS");
        for stream in &prefixed_streams {
            cmd.arg(stream);
        }
        for _ in &prefixed_streams {
            cmd.arg(">"); // 只读取新消息
        }

        let result: Value = cmd
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XREADGROUP: {}", e)))?;

        parse_xread_response(result)
    }

    /// 确认消息
    pub async fn xack(&mut self, stream: &str, ids: &[&str]) -> AppResult<usize> {
        let stream = self.prefixed_stream(stream);

        let mut cmd = redis::cmd("XACK");
        cmd.arg(&stream).arg(&self.group);
        for id in ids {
            cmd.arg(*id);
        }

        let acked: usize = cmd
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XACK: {}", e)))?;

        debug!(stream = %stream, acked, "Messages acknowledged");
        Ok(acked)
    }

    /// 读取待处理消息（用于恢复）
    pub async fn xpending(
        &mut self,
        stream: &str,
        count: usize,
    ) -> AppResult<Vec<PendingMessage>> {
        let stream = self.prefixed_stream(stream);

        let result: Value = redis::cmd("XPENDING")
            .arg(&stream)
            .arg(&self.group)
            .arg("-")
            .arg("+")
            .arg(count)
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XPENDING: {}", e)))?;

        parse_xpending_response(result)
    }

    /// 认领超时的消息
    pub async fn xclaim(
        &mut self,
        stream: &str,
        min_idle_ms: usize,
        ids: &[&str],
    ) -> AppResult<Vec<StreamMessage>> {
        let stream = self.prefixed_stream(stream);

        let mut cmd = redis::cmd("XCLAIM");
        cmd.arg(&stream)
            .arg(&self.group)
            .arg(&self.consumer)
            .arg(min_idle_ms);

        for id in ids {
            cmd.arg(*id);
        }

        let result: Value = cmd
            .query_async(&mut self.conn)
            .await
            .map_err(|e| AppError::internal(format!("Failed to XCLAIM: {}", e)))?;

        parse_xclaim_response(result, &stream)
    }
}

/// 待处理消息信息
#[derive(Debug, Clone)]
pub struct PendingMessage {
    /// 消息 ID
    pub id: String,
    /// 消费者名称
    pub consumer: String,
    /// 空闲时间（毫秒）
    pub idle_ms: u64,
    /// 投递次数
    pub delivery_count: u64,
}

/// 解析 XREAD/XREADGROUP 响应
fn parse_xread_response(value: Value) -> AppResult<Vec<StreamMessage>> {
    let mut messages = Vec::new();

    if let Value::Array(streams) = value {
        for stream_data in streams {
            if let Value::Array(parts) = stream_data {
                if parts.len() >= 2 {
                    let stream_name = match &parts[0] {
                        Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                        _ => continue,
                    };

                    if let Value::Array(entries) = &parts[1] {
                        for entry in entries {
                            if let Value::Array(entry_parts) = entry {
                                if entry_parts.len() >= 2 {
                                    let id = match &entry_parts[0] {
                                        Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                                        _ => continue,
                                    };

                                    let mut fields = HashMap::new();
                                    if let Value::Array(field_values) = &entry_parts[1] {
                                        let mut iter = field_values.iter();
                                        while let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                                            if let (Value::BulkString(k), Value::BulkString(v)) = (key, value) {
                                                fields.insert(
                                                    String::from_utf8_lossy(k).to_string(),
                                                    String::from_utf8_lossy(v).to_string(),
                                                );
                                            }
                                        }
                                    }

                                    messages.push(StreamMessage {
                                        id,
                                        stream: stream_name.clone(),
                                        fields,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// 解析 XPENDING 响应
fn parse_xpending_response(value: Value) -> AppResult<Vec<PendingMessage>> {
    let mut pending = Vec::new();

    if let Value::Array(entries) = value {
        for entry in entries {
            if let Value::Array(parts) = entry {
                if parts.len() >= 4 {
                    let id = match &parts[0] {
                        Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                        _ => continue,
                    };
                    let consumer = match &parts[1] {
                        Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                        _ => continue,
                    };
                    let idle_ms = match &parts[2] {
                        Value::Int(i) => *i as u64,
                        _ => continue,
                    };
                    let delivery_count = match &parts[3] {
                        Value::Int(i) => *i as u64,
                        _ => continue,
                    };

                    pending.push(PendingMessage {
                        id,
                        consumer,
                        idle_ms,
                        delivery_count,
                    });
                }
            }
        }
    }

    Ok(pending)
}

/// 解析 XCLAIM 响应
fn parse_xclaim_response(value: Value, stream: &str) -> AppResult<Vec<StreamMessage>> {
    let mut messages = Vec::new();

    if let Value::Array(entries) = value {
        for entry in entries {
            if let Value::Array(parts) = entry {
                if parts.len() >= 2 {
                    let id = match &parts[0] {
                        Value::BulkString(s) => String::from_utf8_lossy(s).to_string(),
                        _ => continue,
                    };

                    let mut fields = HashMap::new();
                    if let Value::Array(field_values) = &parts[1] {
                        let mut iter = field_values.iter();
                        while let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                            if let (Value::BulkString(k), Value::BulkString(v)) = (key, value) {
                                fields.insert(
                                    String::from_utf8_lossy(k).to_string(),
                                    String::from_utf8_lossy(v).to_string(),
                                );
                            }
                        }
                    }

                    messages.push(StreamMessage {
                        id,
                        stream: stream.to_string(),
                        fields,
                    });
                }
            }
        }
    }

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_message() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), r#"{"key": "value"}"#.to_string());

        let msg = StreamMessage {
            id: "1234-0".to_string(),
            stream: "test-stream".to_string(),
            fields,
        };

        assert_eq!(msg.get("data"), Some(&r#"{"key": "value"}"#.to_string()));

        #[derive(Deserialize)]
        struct TestData {
            key: String,
        }

        let data: TestData = msg.get_json("data").unwrap().unwrap();
        assert_eq!(data.key, "value");
    }

    #[tokio::test]
    #[ignore] // 需要 Redis 实例
    async fn test_stream_producer() {
        let config = RedisConfig::new("redis://127.0.0.1:6379");
        let mut producer = StreamProducer::new(&config).await.unwrap();

        let id = producer
            .xadd("test-stream", &[("message", "hello")])
            .await
            .unwrap();

        assert!(!id.is_empty());

        let len = producer.xlen("test-stream").await.unwrap();
        assert!(len > 0);
    }
}
