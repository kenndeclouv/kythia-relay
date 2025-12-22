use tokio::signal;

/// Wait for shutdown signal (SIGTERM or SIGINT)
pub async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                log::info!("Received SIGTERM, initiating graceful shutdown...");
            }
            _ = sigint.recv() => {
                log::info!("Received SIGINT, initiating graceful shutdown...");
            }
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        log::info!("Received Ctrl+C, initiating graceful shutdown...");
    }
}
