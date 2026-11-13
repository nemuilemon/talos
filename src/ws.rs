use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::ticker::{Tick, parse_tick};

const URL: &str = "wss://stream.binance.com:9443/ws/ethusdt@ticker";

// Connection state
#[derive(Debug, Clone)]
pub enum Status {
    Connecting,
    Live,
    Reconnecting(String), // why the connection dropped
}

// What the WebSocket task sends to the UI side
pub enum WsEvent {
    Status(Status),
    Tick(Tick),
}

pub async fn ws_task(tx: mpsc::Sender<WsEvent>) {
    loop {
        let _ = tx.send(WsEvent::Status(Status::Connecting)).await;

        let reason = match connect_async(URL).await {
            Err(e) => format!("connect failed: {e}"),
            Ok((mut ws, _response)) => {
                let _ = tx.send(WsEvent::Status(Status::Live)).await;
                let mut reason = String::from("stream ended");

                while let Some(msg) = ws.next().await {
                    let msg = match msg {
                        Ok(m) => m,
                        Err(e) => {
                            reason = format!("recv failed: {e}");
                            break;
                        }
                    };
                    match msg {
                        Message::Text(text) => {
                            let tick = match parse_tick(&text) {
                                Ok(t) => t,
                                Err(_) => continue, // skip malformed messages
                            };
                            // The UI has quit and dropped the receiver, so stop this task too
                            if tx.send(WsEvent::Tick(tick)).await.is_err() {
                                return;
                            }
                        }
                        Message::Close(frame) => {
                            reason = format!("closed: {frame:?}");
                            break;
                        }
                        _ => {}
                    }
                }
                reason
            }
        };

        if tx
            .send(WsEvent::Status(Status::Reconnecting(reason)))
            .await
            .is_err()
        {
            return;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
