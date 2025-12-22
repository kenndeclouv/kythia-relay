use crate::metrics::MetricsCollector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Start HTTP server for health and metrics endpoints
pub async fn start_http_server(addr: String, metrics: MetricsCollector) {
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind HTTP server to {}: {}", addr, e);
            return;
        }
    };

    log::info!("📊 HTTP server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            handle_http_request(stream, metrics_clone).await;
        });
    }
}

/// Handle HTTP request
async fn handle_http_request(mut stream: TcpStream, metrics: MetricsCollector) {
    let mut buffer = [0; 1024];

    if let Ok(n) = stream.read(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Parse the request line
        if let Some(first_line) = request.lines().next() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();

            if parts.len() >= 2 {
                let path = parts[1];

                match path {
                    "/health" => {
                        send_response(
                            &mut stream,
                            200,
                            "application/json",
                            r#"{"status":"healthy"}"#,
                        )
                        .await;
                    }
                    "/metrics" => {
                        let snapshot = metrics.snapshot();
                        if let Ok(json) = serde_json::to_string(&snapshot) {
                            send_response(&mut stream, 200, "application/json", &json).await;
                        } else {
                            send_response(&mut stream, 500, "text/plain", "Internal Server Error")
                                .await;
                        }
                    }
                    _ => {
                        send_response(&mut stream, 404, "text/plain", "Not Found").await;
                    }
                }
            }
        }
    }
}

/// Send HTTP response
async fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        content_type,
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}
