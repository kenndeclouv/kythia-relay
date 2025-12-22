use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

// Alias biar gak kepanjangan ngetik
pub type Tx = mpsc::Sender<Message>;

// Protocol JSON
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op", content = "d")]
pub enum SignalingMessage {
    #[serde(rename = "join")]
    Join { room_id: String },
}
