// Module declarations
mod api_keys;
mod auth;
mod config;
mod db;
mod errors;
mod handler;
mod http;
mod metrics;
mod ratelimit;
mod shutdown;
mod state;
mod types;

use api_keys::ApiKeyManager;
use config::Config;
use db::Database;
use metrics::MetricsCollector;
use ratelimit::ClientRateLimiter;
use std::fs;
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

    // Initialize database
    log::info!("🔌 Connecting to database...");
    let database = match Database::new(&config.database_url, config.db_max_connections).await {
        Ok(db) => {
            log::info!("✅ Database connected successfully");
            db
        }
        Err(e) => {
            log::error!("❌ Failed to connect to database: {}", e);
            log::error!("   Make sure MySQL is running and DATABASE_URL is correct");
            std::process::exit(1);
        }
    };

    // Run database migrations
    if let Err(e) = database.migrate().await {
        log::error!("❌ Database migration failed: {}", e);
        std::process::exit(1);
    }

    // Initialize API Key Manager
    let api_key_manager = ApiKeyManager::new(database.clone());

    // Bootstrap master key if needed
    match api_key_manager.bootstrap_master_key().await {
        Ok(Some(master_key)) => {
            log::warn!("🔑 NEW MASTER KEY GENERATED!");
            log::warn!("   Master Key: {}", master_key);
            log::warn!("   Saving to: {}", config.master_key_file);

            // Save to file
            if let Err(e) = fs::write(&config.master_key_file, &master_key) {
                log::error!("❌ Failed to save master key: {}", e);
                log::warn!("   Please save this key manually: {}", master_key);
            } else {
                log::info!("✅ Master key saved to {}", config.master_key_file);
            }

            log::warn!("   ⚠️  IMPORTANT: Save this key securely! It will not be shown again.");
        }
        Ok(None) => {
            log::info!("🔑 Master key already exists");
        }
        Err(e) => {
            log::error!("❌ Failed to bootstrap master key: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize metrics collector
    let metrics = MetricsCollector::new();

    // Always start HTTP server — it hosts both metrics AND API key management
    {
        let http_metrics = metrics.clone();
        let http_api_manager = Some(api_key_manager.clone());
        let http_addr_clone = http_addr.clone();
        tokio::spawn(async move {
            http::start_http_server(http_addr_clone, http_metrics, http_api_manager).await;
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

    log::info!("🚀 Kythia RelayCore listening on: {}", ws_addr);
    log::info!("📊 Metrics available at: http://{}/metrics", http_addr_display);
    log::info!("💚 Health check at: http://{}/health", http_addr_display);
    log::info!(
        "🔐 Authentication: {}",
        if config.auth_enabled {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    log::info!(
        "⚡ Rate limit: {} msg/s per client | Channel buffer: {} | DB pool: {}",
        config.rate_limit_per_second,
        config.channel_buffer_size,
        config.db_max_connections
    );

    // Initialize state
    let peers = state::init();

    // Initialize rate limiter (shared across all connections)
    let rate_limiter = ClientRateLimiter::new(config.rate_limit_per_second);

    // Snapshot config values for use in the accept loop
    let channel_buffer_size = config.channel_buffer_size;

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
                        let rate_limiter_clone = rate_limiter.clone();

                        // Spawn handler for WebSocket connection
                        tokio::spawn(async move {
                            handler::handle_connection(
                                stream,
                                addr,
                                peers_clone,
                                metrics_clone,
                                rate_limiter_clone,
                                channel_buffer_size,
                            )
                            .await;
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
