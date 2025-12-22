use crate::state::PeerMap;
use crate::types::SignalingMessage;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, peers: PeerMap) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Handshake error: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel(500);

    // Task Writer (Background)
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if write.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut current_room: Option<String> = None;

    // Task Reader (Main Loop)
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(action) = serde_json::from_str::<SignalingMessage>(&text) {
                    match action {
                        SignalingMessage::Join { room_id } => {
                            peers
                                .entry(room_id.clone())
                                .or_insert_with(DashMap::new)
                                .insert(addr, tx.clone());

                            current_room = Some(room_id.clone());
                            log::info!("Client {} joined room {}", addr, room_id);
                        }
                    }
                }
            }

            Message::Binary(data) => {
                if let Some(room_id) = &current_room {
                    if let Some(room) = peers.get(room_id) {
                        for client in room.iter() {
                            let target_addr = client.key();
                            let target_tx = client.value();

                            if target_addr != &addr {
                                let _ = target_tx.try_send(Message::Binary(data.clone()));
                            }
                        }
                    }
                }
            }

            Message::Ping(payload) => {
                let _ = tx.try_send(Message::Pong(payload));
            }

            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup Logic
    if let Some(room_id) = current_room {
        let should_remove_room = {
            if let Some(room) = peers.get(&room_id) {
                room.remove(&addr);
                log::info!("Client {} left room {}", addr, room_id);
                room.is_empty()
            } else {
                false
            }
        };

        if should_remove_room {
            peers.remove(&room_id);
            log::info!("Room {} is empty and deleted 🗑️", room_id);
        }
    }
}
