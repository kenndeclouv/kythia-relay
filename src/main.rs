// Module declarations
mod auth;
mod config;
mod errors;
mod handler;
mod http;
mod metrics;
mod ratelimit;
mod shutdown;
mod state;
mod types;

use config::Config;
use metrics::MetricsCollector;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load and validate configuration
    let config = match Config::load() {
        Ok(cfg) => {
            if let Err(e) = cfg.validate() {
                log::error!("❌ Configuration validation failed: {}", e);
                std::process::exit(1);
            }
            cfg
        }
        Err(e) => {
            log::error!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let ws_addr = config.addr();
    let http_addr = config.http_addr();
    let http_addr_display = http_addr.replace("0.0.0.0", "localhost");

    // Initialize metrics collector
    let metrics = if config.metrics_enabled {
        MetricsCollector::new()
    } else {
        MetricsCollector::new() // Still create it, but won't be exposed if disabled
    };

    // Start HTTP server for health and metrics
    if config.metrics_enabled {
        let http_metrics = metrics.clone();
        tokio::spawn(async move {
            http::start_http_server(http_addr, http_metrics).await;
        });
    }

    // Bind WebSocket listener
    let listener = match TcpListener::bind(&ws_addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("❌ Failed to bind to {}: {}", ws_addr, e);
            std::process::exit(1);
        }
    };

    log::info!("🚀 Kythia Nexus Core listening on: {}", ws_addr);
    if config.metrics_enabled {
        log::info!(
            "📊 Metrics available at: http://{}/metrics",
            http_addr_display
        );
        log::info!("💚 Health check at: http://{}/health", http_addr_display);
    }
    log::info!(
        "🔐 Authentication: {}",
        if config.auth_enabled {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );

    // Initialize state
    let peers = state::init();

    // Spawn shutdown listener
    let mut shutdown_handle = tokio::spawn(async move {
        shutdown::wait_for_shutdown().await;
    });

    // Main accept loop
    loop {
        tokio::select! {
            // Accept new connections
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        log::debug!("New connection from: {}", addr);
                        metrics.connection_established();

                        let peers_clone = peers.clone();
                        let metrics_clone = metrics.clone();

                        // Spawn handler for WebSocket connection
                        tokio::spawn(async move {
                            handler::handle_connection(stream, addr, peers_clone, metrics_clone).await;
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept connection: {}", e);
                    }
                }
            }

            // Handle shutdown signal
            _ = &mut shutdown_handle => {
                log::info!("🛑 Shutting down server...");
                break;
            }
        }
    }

    log::info!("✅ Server stopped gracefully");
}
