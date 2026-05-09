use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// Type alias for message sender channel
pub type Tx = mpsc::Sender<Message>;

/// Entry stored per peer in a room: sender channel + connection timestamp
#[derive(Clone)]
pub struct PeerEntry {
    pub tx: Tx,
    pub joined_at: i64,
}

impl PeerEntry {
    pub fn new(tx: Tx) -> Self {
        PeerEntry {
            tx,
            joined_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Type-safe wrapper for room identifiers
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(String);

#[allow(dead_code)]
impl RoomId {
    /// Create a new RoomId
    pub fn new(id: String) -> Self {
        RoomId(id)
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RoomId {
    fn from(s: String) -> Self {
        RoomId(s)
    }
}

impl From<&str> for RoomId {
    fn from(s: &str) -> Self {
        RoomId(s.to_string())
    }
}

/// WebSocket protocol messages
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op", content = "d")]
pub enum SignalingMessage {
    /// Join a room
    #[serde(rename = "join")]
    Join { room_id: String },

    /// Leave the current room
    #[serde(rename = "leave")]
    Leave,

    /// Application-level ping
    #[serde(rename = "ping")]
    Ping,

    /// Application-level pong response
    #[serde(rename = "pong")]
    Pong,

    /// List clients in current room
    #[serde(rename = "list_clients")]
    ListClients,

    /// List all rooms (admin/debug)
    #[serde(rename = "list_rooms")]
    ListRooms,

    /// Error response from server
    #[serde(rename = "error")]
    Error {
        message: String,
        code: Option<String>,
    },

    /// Success response with data
    #[serde(rename = "response")]
    Response { data: ResponseData },
}

/// Response data variants
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ResponseData {
    /// List of clients
    Clients(Vec<ClientInfo>),

    /// List of rooms
    Rooms(Vec<RoomInfo>),

    /// Generic success message
    Success { message: String },
}

/// Client information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    /// Client socket address (anonymized for privacy)
    pub id: String,

    /// When the client joined
    pub joined_at: i64,
}

impl ClientInfo {
    /// Create a new ClientInfo from SocketAddr and pre-computed join timestamp
    pub fn new(addr: SocketAddr, joined_at: i64) -> Self {
        Self {
            id: format!("{}", addr),
            joined_at,
        }
    }
}

/// Room metadata information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    /// Room identifier
    pub room_id: String,

    /// Number of connected clients
    pub client_count: usize,

    /// When the room was created
    pub created_at: i64,

    /// Whether the room is private
    pub is_private: bool,
}
