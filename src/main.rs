// Daftarin module biar kebaca sama compiler
mod config;
mod handler;
mod state;
mod types;

use config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup Logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config::load();
    let addr = config.addr();

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    log::info!("🚀 Kythia Nexus Core (Rust) listening on: {}", addr);

    // Init State
    let peers = state::init();

    while let Ok((stream, addr)) = listener.accept().await {
        log::info!("New connection: {}", addr);

        let peers_clone = peers.clone();

        // Panggil logic dari handler.rs
        tokio::spawn(async move {
            handler::handle_connection(stream, addr, peers_clone).await;
        });
    }
}
