use crate::chat::message::model::{CreateMessageRequest, MessageResponse};
use crate::common::state::AppState;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{Json, Router};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};
use tracing::info;

// TODO: Replace with better data structure
type Tx = Mutex<HashMap<i64, Sender<MessageResponse>>>;

pub struct MessageHandler {
    tx_map: Tx,
}

impl MessageHandler {
    pub fn new() -> Self {
        let tx_map = Mutex::new(HashMap::new());
        Self { tx_map }
    }

    async fn create_message(&self, req: CreateMessageRequest) -> impl IntoResponse {
        let message = MessageResponse {
            id: 0,
            conversation_id: req.conversation_id,
            sender_id: 0,
            text: req.text,
            deleted_at: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let channels_guard = self.tx_map.lock().await;
        if let Some(tx) = channels_guard.get(&message.conversation_id) {
            tx.send(message.clone()).unwrap();
        }

        Json(message)
    }

    async fn publish_message(&self, socket: WebSocket, conversation_id: i64) {
        let mut channels_guard = self.tx_map.lock().await;

        if !channels_guard.contains_key(&conversation_id) {
            let (tx, _) = broadcast::channel::<MessageResponse>(100);
            channels_guard.insert(conversation_id, tx);
        }

        let tx = channels_guard.get_mut(&conversation_id).unwrap();
        let mut rx = tx.subscribe();

        // Early drop to prevent deadlock
        drop(channels_guard);

        let (mut sender, mut receiver) = socket.split();

        // Receive messages from another users, and publish the message to current user
        let mut send_task = tokio::spawn(async move {
            while let Ok(res) = rx.recv().await {
                let text = serde_json::ser::to_string(&res).unwrap();
                if sender.send(ws::Message::Text(text)).await.is_err() {
                    break;
                };
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(ws::Message::Close(_))) = receiver.next().await {
                break;
            }
        });

        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        }

        let mut channels_guard = self.tx_map.lock().await;
        let tx = channels_guard.get(&conversation_id).unwrap();
        if tx.is_empty() {
            channels_guard.remove(&conversation_id);
        }
    }

    pub fn create_route(handler: Arc<Self>, router: Router<AppState>) -> Router<AppState> {
        router
            .route(
                "/ws/conversation/:conversation_id/messages",
                any({
                    let handler = Arc::clone(&handler);
                    |ws: WebSocketUpgrade, Path(conversation_id): Path<i64>| async move {
                        info!("Websocket upgrade requested");
                        ws.on_upgrade(move |socket| async move {
                            handler.publish_message(socket, conversation_id).await
                        })
                    }
                }),
            )
            .route(
                "/api/message",
                post({
                    let handler = Arc::clone(&handler);
                    |Json(req): Json<CreateMessageRequest>| async move {
                        handler.create_message(req).await
                    }
                }),
            )
    }
}
