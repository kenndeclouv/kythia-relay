use crate::types::Tx;
use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};

/// Type alias for the peer map structure
/// Maps RoomID -> (SocketAddr -> SenderChannel)
pub type PeerMap = Arc<DashMap<String, DashMap<SocketAddr, Tx>>>;

/// Initialize the peer map
pub fn init() -> PeerMap {
    Arc::new(DashMap::new())
}
