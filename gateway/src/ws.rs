use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use cuba_auth_core::TokenService;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// WebSocket 状态
#[derive(Clone)]
pub struct WsState {
    pub notify_tx: Arc<broadcast::Sender<String>>,
    pub token_service: TokenService,
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

/// WebSocket 处理器
///
/// 由于浏览器 WebSocket API 不支持自定义 Header，我们通过 query parameter 传递 token。
/// 例如：ws://localhost:8080/ws/events?token=YOUR_JWT_TOKEN
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // 验证 token
    let claims = state
        .token_service
        .validate_token(&query.token)
        .map_err(|e| {
            warn!("WebSocket authentication failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    info!(
        user_id = %claims.sub,
        tenant_id = %claims.tenant_id,
        "WebSocket connection authenticated"
    );

    // Token 验证成功，升级连接
    Ok(ws.on_upgrade(move |socket| {
        handle_socket(socket, state.notify_tx, claims.sub, claims.tenant_id)
    }))
}

async fn handle_socket(
    socket: WebSocket,
    tx: Arc<broadcast::Sender<String>>,
    user_id: String,
    tenant_id: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // 订阅广播通道
    let mut rx = tx.subscribe();

    // 启动一个任务将广播消息推送到 WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 在实际应用中，可以在这里过滤消息，只发送给特定用户/租户
            // 例如：解析消息中的 tenant_id，只发送匹配的消息
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
                Message::Ping(_) => {} // axum/tungstenite handles pong automatically
                _ => {}                // 忽略客户端发送的其他消息
            }
        }
    });

    // 增加连接计数
    metrics::gauge!("gateway.ws.active_connections").increment(1.0);

    // 等待任一任务结束
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // 减少连接计数
    metrics::gauge!("gateway.ws.active_connections").decrement(1.0);

    info!(
        user_id = %user_id,
        tenant_id = %tenant_id,
        "WebSocket disconnected"
    );
}
