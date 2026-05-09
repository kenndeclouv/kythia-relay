use crate::metrics::MetricsCollector;
use crate::ratelimit::ClientRateLimiter;
use crate::state::PeerMap;
use crate::types::{ClientInfo, PeerEntry, ResponseData, RoomInfo, SignalingMessage};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Send a JSON message to the client
fn send_json_response(tx: &mpsc::Sender<Message>, msg: &SignalingMessage) {
    if let Ok(json) = serde_json::to_string(msg) {
        let _ = tx.try_send(Message::Text(json));
    }
}

/// Handle WebSocket connection
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    peers: PeerMap,
    metrics: MetricsCollector,
    rate_limiter: ClientRateLimiter,
    channel_buffer_size: usize,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Handshake error: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel(channel_buffer_size);

    // Writer task (background) — forwards from mpsc channel to WebSocket sink
    let metrics_writer = metrics.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let byte_len = match &message {
                Message::Text(t) => t.len(),
                Message::Binary(b) => b.len(),
                _ => 0,
            };
            if write.send(message).await.is_err() {
                break;
            }
            if byte_len > 0 {
                metrics_writer.message_sent(byte_len);
            }
        }
    });

    let mut current_room: Option<String> = None;

    // Reader task (main loop)
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                let byte_len = text.len();

                // Check rate limit before processing
                if let Err(_) = rate_limiter.check(addr) {
                    metrics.message_dropped();
                    send_json_response(
                        &tx,
                        &SignalingMessage::Error {
                            message: "Rate limit exceeded".to_string(),
                            code: Some("RATE_LIMITED".to_string()),
                        },
                    );
                    continue;
                }

                metrics.message_received(byte_len);

                if let Ok(action) = serde_json::from_str::<SignalingMessage>(&text) {
                    match action {
                        SignalingMessage::Join { room_id } => {
                            // Leave current room if in a different one
                            if let Some(old_room) = &current_room {
                                if old_room != &room_id {
                                    if let Some(room) = peers.get(old_room) {
                                        room.remove(&addr);
                                        if room.is_empty() {
                                            drop(room); // release read guard before write
                                            peers.remove(old_room);
                                            metrics.room_deleted();
                                            log::info!("Room {} is empty and deleted 🗑️", old_room);
                                        }
                                    }
                                }
                            }

                            // Check if this is a brand-new room
                            let is_new_room = !peers.contains_key(&room_id);

                            // Join new room — store PeerEntry (tx + joined_at)
                            peers
                                .entry(room_id.clone())
                                .or_insert_with(DashMap::new)
                                .insert(addr, PeerEntry::new(tx.clone()));

                            current_room = Some(room_id.clone());

                            if is_new_room {
                                metrics.room_created();
                            }

                            log::info!("Client {} joined room {}", addr, room_id);

                            // Send success response
                            send_json_response(
                                &tx,
                                &SignalingMessage::Response {
                                    data: ResponseData::Success {
                                        message: format!("Joined room: {}", room_id),
                                    },
                                },
                            );
                        }

                        SignalingMessage::Leave => {
                            if let Some(room_id) = &current_room {
                                let should_remove_room = {
                                    if let Some(room) = peers.get(room_id) {
                                        room.remove(&addr);
                                        log::info!("Client {} left room {}", addr, room_id);
                                        room.is_empty()
                                    } else {
                                        false
                                    }
                                };

                                if should_remove_room {
                                    peers.remove(room_id);
                                    metrics.room_deleted();
                                    log::info!("Room {} is empty and deleted 🗑️", room_id);
                                }

                                current_room = None;

                                send_json_response(
                                    &tx,
                                    &SignalingMessage::Response {
                                        data: ResponseData::Success {
                                            message: "Left room".to_string(),
                                        },
                                    },
                                );
                            } else {
                                send_json_response(
                                    &tx,
                                    &SignalingMessage::Error {
                                        message: "Not in any room".to_string(),
                                        code: Some("NOT_IN_ROOM".to_string()),
                                    },
                                );
                            }
                        }

                        SignalingMessage::Ping => {
                            send_json_response(&tx, &SignalingMessage::Pong);
                        }

                        SignalingMessage::ListClients => {
                            if let Some(room_id) = &current_room {
                                if let Some(room) = peers.get(room_id) {
                                    let clients: Vec<ClientInfo> = room
                                        .iter()
                                        .map(|entry| {
                                            ClientInfo::new(*entry.key(), entry.value().joined_at)
                                        })
                                        .collect();

                                    send_json_response(
                                        &tx,
                                        &SignalingMessage::Response {
                                            data: ResponseData::Clients(clients),
                                        },
                                    );
                                } else {
                                    send_json_response(
                                        &tx,
                                        &SignalingMessage::Error {
                                            message: "Room not found".to_string(),
                                            code: Some("ROOM_NOT_FOUND".to_string()),
                                        },
                                    );
                                }
                            } else {
                                send_json_response(
                                    &tx,
                                    &SignalingMessage::Error {
                                        message: "Not in any room".to_string(),
                                        code: Some("NOT_IN_ROOM".to_string()),
                                    },
                                );
                            }
                        }

                        SignalingMessage::ListRooms => {
                            let rooms: Vec<RoomInfo> = peers
                                .iter()
                                .map(|entry| RoomInfo {
                                    room_id: entry.key().clone(),
                                    client_count: entry.value().len(),
                                    created_at: 0,     // TODO: Track creation time per room
                                    is_private: false, // TODO: Implement private rooms
                                })
                                .collect();

                            send_json_response(
                                &tx,
                                &SignalingMessage::Response {
                                    data: ResponseData::Rooms(rooms),
                                },
                            );
                        }

                        // Server-to-client messages — ignore if received from client
                        SignalingMessage::Pong
                        | SignalingMessage::Error { .. }
                        | SignalingMessage::Response { .. } => {
                            log::warn!("Received server-side message from client: {:?}", action);
                        }
                    }
                }
            }

            Message::Binary(data) => {
                // Rate-limit binary relay too
                if let Err(_) = rate_limiter.check(addr) {
                    metrics.message_dropped();
                    continue;
                }

                let byte_len = data.len();
                metrics.message_received(byte_len);

                if let Some(room_id) = &current_room {
                    if let Some(room) = peers.get(room_id) {
                        for client in room.iter() {
                            let target_addr = client.key();
                            let peer = client.value();

                            if target_addr != &addr {
                                if peer.tx.try_send(Message::Binary(data.clone())).is_err() {
                                    metrics.message_dropped();
                                }
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

    // Cleanup on disconnect
    metrics.connection_closed();
    rate_limiter.remove(&addr);

    if let Some(room_id) = current_room {
        let should_remove_room = {
            if let Some(room) = peers.get(&room_id) {
                room.remove(&addr);
                log::info!("Client {} disconnected from room {}", addr, room_id);
                room.is_empty()
            } else {
                false
            }
        };

        if should_remove_room {
            peers.remove(&room_id);
            metrics.room_deleted();
            log::info!("Room {} is empty and deleted 🗑️", room_id);
        }
    }
}
