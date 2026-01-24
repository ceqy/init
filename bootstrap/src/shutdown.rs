//! Graceful Shutdown

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

/// Shutdown 控制器
#[derive(Clone)]
pub struct ShutdownController {
    notify: Arc<Notify>,
}

impl ShutdownController {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    /// 触发关闭
    pub fn shutdown(&self) {
        info!("Triggering shutdown");
        self.notify.notify_waiters();
    }

    /// 等待关闭信号
    pub fn wait(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.notify.notified().await;
        })
    }

    /// 创建一个可以等待关闭的 future
    pub fn shutdown_signal(&self) -> impl Future<Output = ()> + Send + '_ {
        async move {
            self.notify.notified().await;
        }
    }
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::new()
    }
}

/// 运行带有 graceful shutdown 的任务
pub async fn run_with_shutdown<F, Fut>(
    shutdown: ShutdownController,
    task: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    tokio::select! {
        result = task() => result,
        _ = shutdown.wait() => {
            info!("Task cancelled due to shutdown");
            Ok(())
        }
    }
}
