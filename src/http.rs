use crate::api_keys::{ApiKeyManager, CreateApiKeyRequest};
use crate::metrics::MetricsCollector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Start HTTP server for health, metrics, and API key management
pub async fn start_http_server(
    addr: String,
    metrics: MetricsCollector,
    api_key_manager: Option<ApiKeyManager>,
) {
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
        let api_manager_clone = api_key_manager.clone();
        tokio::spawn(async move {
            handle_http_request(stream, metrics_clone, api_manager_clone).await;
        });
    }
}

/// Handle HTTP request
async fn handle_http_request(
    mut stream: TcpStream,
    metrics: MetricsCollector,
    api_key_manager: Option<ApiKeyManager>,
) {
    let mut buffer = vec![0; 4096];

    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == 0 => return,
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse request line
    let mut lines = request.lines();
    let request_line = match lines.next() {
        Some(line) => line,
        None => {
            send_response(&mut stream, 400, "text/plain", "Bad Request").await;
            return;
        }
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(&mut stream, 400, "text/plain", "Bad Request").await;
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // Extract Authorization header
    let auth_token = extract_auth_token(&request);

    // Route requests
    match (method, path) {
        ("GET", "/health") => {
            send_response(
                &mut stream,
                200,
                "application/json",
                r#"{"status":"healthy"}"#,
            )
            .await;
        }
        ("GET", "/metrics") => {
            let snapshot = metrics.snapshot();
            match serde_json::to_string(&snapshot) {
                Ok(json) => send_response(&mut stream, 200, "application/json", &json).await,
                Err(_) => {
                    send_response(&mut stream, 500, "text/plain", "Internal Server Error").await
                }
            }
        }
        _ if path.starts_with("/api/v1/keys") => {
            if let Some(manager) = api_key_manager {
                handle_api_keys_route(&mut stream, method, path, auth_token, manager).await;
            } else {
                send_response(
                    &mut stream,
                    503,
                    "application/json",
                    r#"{"error":"API key management not available"}"#,
                )
                .await;
            }
        }
        _ => {
            send_response(&mut stream, 404, "text/plain", "Not Found").await;
        }
    }
}

/// Handle API keys routes
async fn handle_api_keys_route(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    auth_token: Option<String>,
    manager: ApiKeyManager,
) {
    // Verify authorization
    let api_key = match auth_token {
        Some(key) => key,
        None => {
            send_json_error(stream, 401, "Missing authorization header").await;
            return;
        }
    };

    // Verify the API key
    match manager.verify_key(&api_key).await {
        Ok(true) => {}
        Ok(false) => {
            send_json_error(stream, 401, "Invalid or inactive API key").await;
            return;
        }
        Err(e) => {
            log::error!("Failed to verify API key: {}", e);
            send_json_error(stream, 500, "Internal server error").await;
            return;
        }
    }

    // Route to specific handlers
    match (method, path) {
        ("POST", "/api/v1/keys") => create_api_key(stream, manager).await,
        ("GET", "/api/v1/keys") => list_api_keys(stream, manager).await,
        ("GET", path) if path.starts_with("/api/v1/keys/") => {
            if let Some(id_str) = path.strip_prefix("/api/v1/keys/") {
                if let Ok(id) = id_str.parse::<i32>() {
                    get_api_key(stream, id, manager).await;
                } else {
                    send_json_error(stream, 400, "Invalid key ID").await;
                }
            } else {
                send_json_error(stream, 404, "Not found").await;
            }
        }
        ("PATCH", path) if path.contains("/activate") => {
            if let Some(id) = extract_key_id(path) {
                activate_api_key(stream, id, manager).await;
            } else {
                send_json_error(stream, 400, "Invalid key ID").await;
            }
        }
        ("PATCH", path) if path.contains("/deactivate") => {
            if let Some(id) = extract_key_id(path) {
                deactivate_api_key(stream, id, manager).await;
            } else {
                send_json_error(stream, 400, "Invalid key ID").await;
            }
        }
        ("DELETE", path) if path.starts_with("/api/v1/keys/") => {
            if let Some(id) = extract_key_id(path) {
                delete_api_key(stream, id, manager).await;
            } else {
                send_json_error(stream, 400, "Invalid key ID").await;
            }
        }
        _ => {
            send_json_error(stream, 404, "Not found").await;
        }
    }
}

/// Extract API key ID from path
fn extract_key_id(path: &str) -> Option<i32> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 4 {
        parts[3].parse::<i32>().ok()
    } else {
        None
    }
}

/// Create a new API key
async fn create_api_key(stream: &mut TcpStream, manager: ApiKeyManager) {
    // For simplicity, create with default name (in production, parse request body)
    let request = CreateApiKeyRequest {
        name: "API Key".to_string(),
        metadata: None,
    };

    match manager
        .create_key(request.name, false, request.metadata)
        .await
    {
        Ok(key_with_secret) => match serde_json::to_string(&key_with_secret) {
            Ok(json) => send_response(stream, 201, "application/json", &json).await,
            Err(_) => send_json_error(stream, 500, "Failed to serialize response").await,
        },
        Err(e) => {
            log::error!("Failed to create API key: {}", e);
            send_json_error(stream, 500, "Failed to create API key").await;
        }
    }
}

/// List all API keys
async fn list_api_keys(stream: &mut TcpStream, manager: ApiKeyManager) {
    match manager.list_keys().await {
        Ok(keys) => {
            let response = serde_json::json!({"keys": keys});
            match serde_json::to_string(&response) {
                Ok(json) => send_response(stream, 200, "application/json", &json).await,
                Err(_) => send_json_error(stream, 500, "Failed to serialize response").await,
            }
        }
        Err(e) => {
            log::error!("Failed to list API keys: {}", e);
            send_json_error(stream, 500, "Failed to list API keys").await;
        }
    }
}

/// Get a specific API key
async fn get_api_key(stream: &mut TcpStream, id: i32, manager: ApiKeyManager) {
    match manager.get_key(id).await {
        Ok(Some(key)) => match serde_json::to_string(&key) {
            Ok(json) => send_response(stream, 200, "application/json", &json).await,
            Err(_) => send_json_error(stream, 500, "Failed to serialize response").await,
        },
        Ok(None) => {
            send_json_error(stream, 404, "API key not found").await;
        }
        Err(e) => {
            log::error!("Failed to get API key: {}", e);
            send_json_error(stream, 500, "Failed to get API key").await;
        }
    }
}

/// Activate an API key
async fn activate_api_key(stream: &mut TcpStream, id: i32, manager: ApiKeyManager) {
    match manager.activate_key(id).await {
        Ok(_) => {
            send_response(
                stream,
                200,
                "application/json",
                r#"{"message":"API key activated"}"#,
            )
            .await;
        }
        Err(e) => {
            log::error!("Failed to activate API key: {}", e);
            send_json_error(stream, 500, "Failed to activate API key").await;
        }
    }
}

/// Deactivate an API key
async fn deactivate_api_key(stream: &mut TcpStream, id: i32, manager: ApiKeyManager) {
    match manager.deactivate_key(id).await {
        Ok(_) => {
            send_response(
                stream,
                200,
                "application/json",
                r#"{"message":"API key deactivated"}"#,
            )
            .await;
        }
        Err(e) => {
            log::error!("Failed to deactivate API key: {}", e);
            send_json_error(stream, 500, &format!("Failed to deactivate: {}", e)).await;
        }
    }
}

/// Delete an API key
async fn delete_api_key(stream: &mut TcpStream, id: i32, manager: ApiKeyManager) {
    match manager.delete_key(id).await {
        Ok(_) => {
            send_response(
                stream,
                200,
                "application/json",
                r#"{"message":"API key deleted"}"#,
            )
            .await;
        }
        Err(e) => {
            log::error!("Failed to delete API key: {}", e);
            send_json_error(stream, 500, &format!("Failed to delete: {}", e)).await;
        }
    }
}

/// Extract authorization token from request headers
fn extract_auth_token(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("authorization:") {
            let auth_value = line.split(':').nth(1)?.trim();
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    None
}

/// Send JSON error response
async fn send_json_error(stream: &mut TcpStream, status: u16, message: &str) {
    let json = format!(r#"{{"error":"{}"}}"#, message);
    send_response(stream, status, "application/json", &json).await;
}

/// Send HTTP response
async fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
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
