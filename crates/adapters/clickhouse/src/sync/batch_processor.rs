//! 批量处理器模块
//!
//! 提供通用的批量数据处理功能

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use clickhouse::Row;
use errors::AppResult;
use serde::Serialize;
use tracing::{debug, error, info};

use crate::batch::BatchWriter;
use crate::client::ClickHousePool;
use crate::config::BatchConfig;

/// 批量处理器配置
#[derive(Debug, Clone)]
pub struct BatchProcessorConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 处理间隔
    pub interval: Duration,
    /// 并发数
    pub concurrency: usize,
    /// 失败重试次数
    pub max_retries: u32,
}

impl Default for BatchProcessorConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            interval: Duration::from_secs(5),
            concurrency: 4,
            max_retries: 3,
        }
    }
}

impl BatchProcessorConfig {
    /// 创建新的配置
    pub fn new(batch_size: usize, interval: Duration) -> Self {
        Self {
            batch_size,
            interval,
            ..Default::default()
        }
    }

    /// 设置并发数
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// 设置重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// 批量处理器状态
#[derive(Debug, Clone, Default)]
pub struct BatchProcessorStats {
    /// 已处理批次数
    pub batches_processed: u64,
    /// 已处理记录数
    pub records_processed: u64,
    /// 失败批次数
    pub batches_failed: u64,
    /// 失败记录数
    pub records_failed: u64,
}

/// 批量处理器
///
/// 通用的批量数据处理器，支持从任意数据源读取并写入 ClickHouse
pub struct BatchProcessor<T: Row + Serialize + Send + Sync + 'static> {
    #[allow(dead_code)]
    pool: Arc<ClickHousePool>,
    batch_writer: Arc<BatchWriter<T>>,
    config: BatchProcessorConfig,
    stats: Arc<parking_lot::Mutex<BatchProcessorStats>>,
}

impl<T: Row + Serialize + Send + Sync + Clone + 'static> BatchProcessor<T> {
    /// 创建新的批量处理器
    pub fn new(
        pool: Arc<ClickHousePool>,
        table: impl Into<String>,
        config: BatchProcessorConfig,
    ) -> Self {
        let batch_config = BatchConfig::new(config.batch_size, config.interval);
        let batch_writer = Arc::new(BatchWriter::new(pool.clone(), table, batch_config));

        Self {
            pool,
            batch_writer,
            config,
            stats: Arc::new(parking_lot::Mutex::new(BatchProcessorStats::default())),
        }
    }

    /// 处理单批数据
    pub async fn process_batch(&self, records: Vec<T>) -> AppResult<u64> {
        if records.is_empty() {
            return Ok(0);
        }

        let count = records.len() as u64;

        match self.batch_writer.insert_many(records).await {
            Ok(()) => {
                let mut stats = self.stats.lock();
                stats.batches_processed += 1;
                stats.records_processed += count;
                Ok(count)
            }
            Err(e) => {
                let mut stats = self.stats.lock();
                stats.batches_failed += 1;
                stats.records_failed += count;
                Err(e)
            }
        }
    }

    /// 启动持续处理
    ///
    /// 从数据源函数持续获取数据并处理
    pub async fn start<F, Fut>(&self, mut fetch_fn: F) -> AppResult<()>
    where
        F: FnMut(usize) -> Fut + Send,
        Fut: Future<Output = AppResult<Vec<T>>> + Send,
    {
        info!(
            batch_size = self.config.batch_size,
            interval_secs = self.config.interval.as_secs(),
            "Starting batch processor"
        );

        loop {
            // 获取一批数据
            match fetch_fn(self.config.batch_size).await {
                Ok(records) => {
                    if records.is_empty() {
                        debug!("No records to process, waiting...");
                        tokio::time::sleep(self.config.interval).await;
                        continue;
                    }

                    let count = records.len();
                    debug!(count, "Processing batch");

                    if let Err(e) = self.process_batch(records).await {
                        error!(error = %e, "Failed to process batch");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to fetch records");
                    tokio::time::sleep(self.config.interval).await;
                }
            }

            // 等待下一个间隔
            tokio::time::sleep(self.config.interval).await;
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> BatchProcessorStats {
        self.stats.lock().clone()
    }

    /// 刷新缓冲区
    pub async fn flush(&self) -> AppResult<()> {
        self.batch_writer.flush().await
    }

    /// 获取批量写入器
    pub fn batch_writer(&self) -> Arc<BatchWriter<T>> {
        self.batch_writer.clone()
    }
}

/// 并行批量处理器
///
/// 支持多个 worker 并行处理
pub struct ParallelBatchProcessor<T: Row + Serialize + Send + Sync + 'static> {
    pool: Arc<ClickHousePool>,
    table: String,
    config: BatchProcessorConfig,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Row + Serialize + Send + Sync + Clone + 'static> ParallelBatchProcessor<T> {
    /// 创建新的并行批量处理器
    pub fn new(
        pool: Arc<ClickHousePool>,
        table: impl Into<String>,
        config: BatchProcessorConfig,
    ) -> Self {
        Self {
            pool,
            table: table.into(),
            config,
            _marker: std::marker::PhantomData,
        }
    }

    /// 启动并行处理
    pub async fn start<F, Fut>(
        &self,
        fetch_fn: F,
    ) -> AppResult<Vec<tokio::task::JoinHandle<()>>>
    where
        F: Fn(usize, usize) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = AppResult<Vec<T>>> + Send + 'static,
    {
        let mut handles = Vec::with_capacity(self.config.concurrency);

        for worker_id in 0..self.config.concurrency {
            let pool = self.pool.clone();
            let table = self.table.clone();
            let config = self.config.clone();
            let fetch = fetch_fn.clone();

            let handle = tokio::spawn(async move {
                let processor = BatchProcessor::new(pool, table, config.clone());

                info!(worker_id, "Starting parallel batch processor worker");

                loop {
                    match fetch(worker_id, config.batch_size).await {
                        Ok(records) => {
                            if records.is_empty() {
                                tokio::time::sleep(config.interval).await;
                                continue;
                            }

                            if let Err(e) = processor.process_batch(records).await {
                                error!(worker_id, error = %e, "Worker failed to process batch");
                            }
                        }
                        Err(e) => {
                            error!(worker_id, error = %e, "Worker failed to fetch records");
                            tokio::time::sleep(config.interval).await;
                        }
                    }

                    tokio::time::sleep(config.interval).await;
                }
            });

            handles.push(handle);
        }

        Ok(handles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_config() {
        let config = BatchProcessorConfig::new(5000, Duration::from_secs(10))
            .with_concurrency(8)
            .with_max_retries(5);

        assert_eq!(config.batch_size, 5000);
        assert_eq!(config.interval, Duration::from_secs(10));
        assert_eq!(config.concurrency, 8);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_batch_processor_stats() {
        let stats = BatchProcessorStats {
            batches_processed: 10,
            records_processed: 10000,
            batches_failed: 1,
            records_failed: 500,
        };

        assert_eq!(stats.batches_processed, 10);
        assert_eq!(stats.records_processed, 10000);
    }
}
