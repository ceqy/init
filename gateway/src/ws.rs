use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::middleware::AuthToken;

/// WebSocket 处理器
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<broadcast::Sender<String>>>,
    // 鉴权：只有通过 AuthToken 中间件验证的用户才能连接
    // 注意：在实际浏览器中，WebSocket 通常不支持直接设置自定义 Header (Authorization)。
    // 这里假设客户端通过 Query Param 传递 token，或者在建立连接前已经通过 Cookie 认证。
    // 为了简单起见，这里复用 AuthToken，假设它能从 Header 提取。
    // 如果浏览器无法发送 Header，需要改为从 Query Param 提取 Token 并手动验证。
    _user: AuthToken, 
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, tx: Arc<broadcast::Sender<String>>) {
    let (mut sender, mut receiver) = socket.split();

    // 订阅广播通道
    let mut rx = tx.subscribe();

    // 启动一个任务将广播消息推送到 WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 在实际应用中，可以在这里过滤消息，只发送给特定用户/租户
            if let Err(e) = sender.send(Message::Text(msg.into())).await {
                warn!("Failed to send message to websocket: {}", e);
                break;
            }
        }
    });

    // 启动一个任务接收 WebSocket 消息 (例如心跳 pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(_) => {}, // axum/tungstenite handles pong automatically usually
                _ => {}, // 忽略客户端发送的其他消息
            }
        }
    });

    // 等待任一任务结束
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("WebSocket disconnected");
}
