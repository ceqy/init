//! 批量写入缓冲模块
//!
//! 提供自动按大小或时间触发写入、内存控制、后台异步刷新

use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use clickhouse::Row;
use errors::{AppError, AppResult};
use parking_lot::Mutex;
use serde::Serialize;
use tokio::sync::Notify;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use crate::client::ClickHousePool;
use crate::config::BatchConfig;

/// 批量写入器状态
#[derive(Debug, Clone)]
pub struct BatchWriterStatus {
    /// 当前缓冲区大小
    pub buffer_size: usize,
    /// 已写入总行数
    pub total_written: u64,
    /// 写入批次数
    pub batch_count: u64,
    /// 失败批次数
    pub failed_batches: u64,
    /// 估计内存使用（字节）
    pub estimated_memory: usize,
}

/// 批量写入器内部状态
struct BatchWriterInner<T> {
    buffer: Vec<T>,
    last_flush: Instant,
    total_written: u64,
    batch_count: u64,
    failed_batches: u64,
}

/// 批量写入器
///
/// 自动按大小或时间触发写入到 ClickHouse
pub struct BatchWriter<T: Row + Serialize + Send + Sync + 'static> {
    pool: Arc<ClickHousePool>,
    table: String,
    config: BatchConfig,
    inner: Arc<Mutex<BatchWriterInner<T>>>,
    flush_notify: Arc<Notify>,
    _marker: PhantomData<T>,
}

impl<T: Row + Serialize + Send + Sync + 'static> BatchWriter<T> {
    /// 创建新的批量写入器
    pub fn new(pool: Arc<ClickHousePool>, table: impl Into<String>, config: BatchConfig) -> Self {
        let inner = Arc::new(Mutex::new(BatchWriterInner {
            buffer: Vec::with_capacity(config.size),
            last_flush: Instant::now(),
            total_written: 0,
            batch_count: 0,
            failed_batches: 0,
        }));

        Self {
            pool,
            table: table.into(),
            config,
            inner,
            flush_notify: Arc::new(Notify::new()),
            _marker: PhantomData,
        }
    }

    /// 插入单条记录
    pub async fn insert(&self, row: T) -> AppResult<()> {
        let should_flush = {
            let mut inner = self.inner.lock();
            inner.buffer.push(row);
            self.should_flush(&inner)
        };

        if should_flush {
            self.flush().await?;
        }

        Ok(())
    }

    /// 插入多条记录
    pub async fn insert_many(&self, rows: Vec<T>) -> AppResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        let should_flush = {
            let mut inner = self.inner.lock();
            inner.buffer.extend(rows);
            self.should_flush(&inner)
        };

        if should_flush {
            self.flush().await?;
        }

        Ok(())
    }

    /// 检查是否应该刷新
    fn should_flush(&self, inner: &BatchWriterInner<T>) -> bool {
        // 按大小触发
        if inner.buffer.len() >= self.config.size {
            return true;
        }

        // 按时间触发
        if inner.last_flush.elapsed() >= self.config.timeout && !inner.buffer.is_empty() {
            return true;
        }

        false
    }

    /// 刷新缓冲区到 ClickHouse
    pub async fn flush(&self) -> AppResult<()> {
        let rows = {
            let mut inner = self.inner.lock();
            if inner.buffer.is_empty() {
                return Ok(());
            }
            inner.last_flush = Instant::now();
            mem::take(&mut inner.buffer)
        };

        let count = rows.len();
        debug!(table = %self.table, count, "Flushing batch to ClickHouse");

        match self.write_batch(rows).await {
            Ok(()) => {
                let mut inner = self.inner.lock();
                inner.total_written += count as u64;
                inner.batch_count += 1;
                debug!(
                    table = %self.table,
                    count,
                    total = inner.total_written,
                    "Batch written successfully"
                );
                Ok(())
            }
            Err(e) => {
                let mut inner = self.inner.lock();
                inner.failed_batches += 1;
                error!(
                    table = %self.table,
                    count,
                    error = %e,
                    "Failed to write batch"
                );
                Err(e)
            }
        }
    }

    /// 写入批量数据到 ClickHouse
    async fn write_batch(&self, rows: Vec<T>) -> AppResult<()> {
        let client = self.pool.get().await?;

        let mut insert = client
            .insert(&self.table)
            .map_err(|e| AppError::database(format!("Failed to create insert: {}", e)))?;

        for row in rows {
            insert
                .write(&row)
                .await
                .map_err(|e| AppError::database(format!("Failed to write row: {}", e)))?;
        }

        insert
            .end()
            .await
            .map_err(|e| AppError::database(format!("Failed to end insert: {}", e)))?;

        Ok(())
    }

    /// 获取状态
    pub fn status(&self) -> BatchWriterStatus {
        let inner = self.inner.lock();
        BatchWriterStatus {
            buffer_size: inner.buffer.len(),
            total_written: inner.total_written,
            batch_count: inner.batch_count,
            failed_batches: inner.failed_batches,
            estimated_memory: inner.buffer.len() * mem::size_of::<T>(),
        }
    }

    /// 获取缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.inner.lock().buffer.len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.lock().buffer.is_empty()
    }

    /// 启动后台刷新任务
    pub fn start_background_flush(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let timeout = self.config.timeout;

        tokio::spawn(async move {
            info!(
                table = %self.table,
                timeout_secs = timeout.as_secs(),
                "Starting background batch flush task"
            );

            loop {
                tokio::select! {
                    _ = tokio::time::sleep(timeout) => {
                        if let Err(e) = self.flush().await {
                            warn!(
                                table = %self.table,
                                error = %e,
                                "Background flush failed"
                            );
                        }
                    }
                    _ = self.flush_notify.notified() => {
                        // 收到通知，立即刷新
                        if let Err(e) = self.flush().await {
                            warn!(
                                table = %self.table,
                                error = %e,
                                "Notified flush failed"
                            );
                        }
                    }
                }
            }
        })
    }

    /// 通知后台任务立即刷新
    pub fn notify_flush(&self) {
        self.flush_notify.notify_one();
    }
}

impl<T: Row + Serialize + Send + Sync + 'static> Drop for BatchWriter<T> {
    fn drop(&mut self) {
        let inner = self.inner.lock();
        if !inner.buffer.is_empty() {
            warn!(
                table = %self.table,
                remaining = inner.buffer.len(),
                "BatchWriter dropped with unflushed data"
            );
        }
    }
}

/// 创建批量写入器的工厂函数
pub fn create_batch_writer<T: Row + Serialize + Send + Sync + 'static>(
    pool: Arc<ClickHousePool>,
    table: impl Into<String>,
) -> BatchWriter<T> {
    let config = BatchConfig::from_clickhouse_config(pool.config());
    BatchWriter::new(pool, table, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClickHouseConfig;
    use clickhouse::Row;
    use serde::Serialize;
    use std::time::Duration;

    #[derive(Debug, Clone, Row, Serialize)]
    struct TestRow {
        id: u64,
        name: String,
    }

    #[test]
    fn test_batch_writer_status() {
        let config = ClickHouseConfig::new("http://localhost:8123", "test");
        let pool = Arc::new(ClickHousePool::new(config).unwrap());
        let batch_config = BatchConfig::new(100, Duration::from_secs(5));
        let writer: BatchWriter<TestRow> =
            BatchWriter::new(pool, "test_table", batch_config);

        let status = writer.status();
        assert_eq!(status.buffer_size, 0);
        assert_eq!(status.total_written, 0);
        assert_eq!(status.batch_count, 0);
    }

    #[test]
    fn test_batch_config() {
        let config = BatchConfig::new(5000, Duration::from_secs(10))
            .with_max_memory(50 * 1024 * 1024);

        assert_eq!(config.size, 5000);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_memory, 50 * 1024 * 1024);
    }
}
