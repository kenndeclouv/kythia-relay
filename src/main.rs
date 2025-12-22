use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

type Tx = mpsc::UnboundedSender<Message>;

type PeerMap = Arc<DashMap<String, DashMap<SocketAddr, Tx>>>;

#[derive(Serialize, Deserialize, Debug)]
struct SignalingMessage {
    op: String,
    d: Option<AuthData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthData {
    room_id: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    log::info!("🚀 Kythia Nexus Core (Rust) listening on: {}", addr);

    let peers: PeerMap = Arc::new(DashMap::new());

    while let Ok((stream, addr)) = listener.accept().await {
        log::info!("New connection: {}", addr);
        let peers_clone = peers.clone();

        tokio::spawn(handle_connection(stream, addr, peers_clone));
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, peers: PeerMap) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Error during handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if write.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut current_room: Option<String> = None;

    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<SignalingMessage>(&text) {
                    if parsed.op == "join" {
                        if let Some(data) = parsed.d {
                            let room_id = data.room_id.clone();

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
                                let _ = target_tx.send(Message::Binary(data.clone()));
                            }
                        }
                    }
                }
            }

            Message::Close(_) => break,
            _ => {}
        }
    }

    if let Some(room_id) = current_room {
        if let Some(room) = peers.get(&room_id) {
            room.remove(&addr);
            log::info!("Client {} left room {}", addr, room_id);
        }
    }
}
