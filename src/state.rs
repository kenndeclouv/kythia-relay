use crate::types::PeerEntry;
use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};

/// Type alias for the peer map structure
/// Maps RoomID -> (SocketAddr -> PeerEntry)
pub type PeerMap = Arc<DashMap<String, DashMap<SocketAddr, PeerEntry>>>;

/// Initialize the peer map
pub fn init() -> PeerMap {
    Arc::new(DashMap::new())
}
