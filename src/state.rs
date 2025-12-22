use crate::types::Tx;
use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};

// Kita bungkus PeerMap biar rapi
// RoomID -> (UserAddr -> SenderChannel)
pub type PeerMap = Arc<DashMap<String, DashMap<SocketAddr, Tx>>>;

// Helper buat inisialisasi (Optional, biar main.rs bersih)
pub fn init() -> PeerMap {
    Arc::new(DashMap::new())
}
