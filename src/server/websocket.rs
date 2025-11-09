use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

use crate::server::models::FileEventType;
use crate::server::state::AppState;

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    info!("WebSocket connection request");

    let (res, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    // Subscribe to file change events
    let mut event_rx = state.event_tx.subscribe();

    // Spawn task to forward events to WebSocket
    actix_web::rt::spawn(async move {
        let close_reason = loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                Some(Ok(msg)) = stream.next() => {
                    match msg {
                        Message::Text(text) => {
                            // Handle client messages (e.g., filter events)
                            if let Ok(filter) = serde_json::from_str::<EventFilter>(&text.to_string()) {
                                info!("Received filter: {:?}", filter);
                                // Apply filter logic (for future enhancement)
                            }
                        }
                        Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() {
                                break None;
                            }
                        }
                        Message::Close(reason) => {
                            break reason;
                        }
                        _ => {}
                    }
                }
                // Forward file change events to client
                Ok(event) = event_rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&event) {
                        if session.text(json).await.is_err() {
                            break None;
                        }
                    }
                }
                else => break None
            }
        };

        let _ = session.close(close_reason).await;
        info!("WebSocket connection closed");
    });

    Ok(res)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EventFilter {
    paths: Option<Vec<PathBuf>>,
    event_types: Option<Vec<FileEventType>>,
}
