///! Kythia Relay Benchmark Tool
///!
///! Spins up N concurrent WebSocket clients that join a shared room,
///! then each sends M binary messages. Measures round-trip latency (RTT)
///! and throughput (messages/sec).
///!
///! Usage:
///!   cargo build --release --bin bench
///!   ./target/release/bench --url ws://127.0.0.1:8080 --clients 50 --messages 200
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug)]
struct BenchConfig {
    url: String,
    clients: usize,
    messages_per_client: usize,
    payload_size: usize,
    room_id: String,
}

impl Default for BenchConfig {
    fn default() -> Self {
        BenchConfig {
            url: "ws://127.0.0.1:8080".to_string(),
            clients: 20,
            messages_per_client: 100,
            payload_size: 512,
            room_id: "bench-room".to_string(),
        }
    }
}

fn parse_args() -> BenchConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut cfg = BenchConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" => {
                if i + 1 < args.len() {
                    cfg.url = args[i + 1].clone();
                    i += 1;
                }
            }
            "--clients" | "-c" => {
                if i + 1 < args.len() {
                    cfg.clients = args[i + 1].parse().unwrap_or(cfg.clients);
                    i += 1;
                }
            }
            "--messages" | "-m" => {
                if i + 1 < args.len() {
                    cfg.messages_per_client = args[i + 1].parse().unwrap_or(cfg.messages_per_client);
                    i += 1;
                }
            }
            "--size" | "-s" => {
                if i + 1 < args.len() {
                    cfg.payload_size = args[i + 1].parse().unwrap_or(cfg.payload_size);
                    i += 1;
                }
            }
            "--room" | "-r" => {
                if i + 1 < args.len() {
                    cfg.room_id = args[i + 1].clone();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Kythia Relay Benchmark");
                println!();
                println!("USAGE:");
                println!("  bench [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  --url <URL>          WebSocket URL (default: ws://127.0.0.1:8080)");
                println!("  --clients <N>        Number of concurrent clients (default: 20)");
                println!("  --messages <N>       Messages per client (default: 100)");
                println!("  --size <BYTES>       Binary payload size in bytes (default: 512)");
                println!("  --room <ID>          Room ID to use (default: bench-room)");
                println!("  --help               Show this help");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    cfg
}

/// Percentile calculation (input must be sorted)
fn percentile(sorted: &[u128], pct: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((pct / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[derive(Debug)]
struct ClientResult {
    messages_sent: usize,
    messages_received: usize,
    latencies_us: Vec<u128>, // microseconds
    error: Option<String>,
}

async fn run_client(
    url: String,
    room_id: String,
    messages: usize,
    payload_size: usize,
) -> ClientResult {
    let connect_result = timeout(Duration::from_secs(10), connect_async(&url)).await;

    let ws_stream = match connect_result {
        Ok(Ok((stream, _))) => stream,
        Ok(Err(e)) => {
            return ClientResult {
                messages_sent: 0,
                messages_received: 0,
                latencies_us: vec![],
                error: Some(format!("Connect error: {}", e)),
            };
        }
        Err(_) => {
            return ClientResult {
                messages_sent: 0,
                messages_received: 0,
                latencies_us: vec![],
                error: Some("Connection timeout".to_string()),
            };
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Join room
    let join_msg = serde_json::json!({"op": "join", "d": {"room_id": room_id}}).to_string();
    if write.send(Message::Text(join_msg)).await.is_err() {
        return ClientResult {
            messages_sent: 0,
            messages_received: 0,
            latencies_us: vec![],
            error: Some("Failed to send join message".to_string()),
        };
    }

    // Wait for join confirmation
    let _ = timeout(Duration::from_secs(5), read.next()).await;

    // Build payload: 8 bytes timestamp prefix + padding
    let padding = vec![0xABu8; payload_size.saturating_sub(8)];
    let mut latencies_us = Vec::with_capacity(messages);
    let mut messages_sent = 0usize;
    let mut messages_received = 0usize;

    for _ in 0..messages {
        // Embed send timestamp in first 8 bytes (little-endian u64 microseconds)
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();

        let mut payload = (wall as u64).to_le_bytes().to_vec();
        payload.extend_from_slice(&padding);

        let _ = write.send(Message::Binary(payload)).await;
        messages_sent += 1;

        // Wait for echo back from another client (or self if alone — won't happen in relay)
        // With 1 receiver, we use a short timeout
        match timeout(Duration::from_millis(500), read.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                let recv_wall = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros();

                if data.len() >= 8 {
                    let sent_ts = u64::from_le_bytes(data[..8].try_into().unwrap_or([0u8; 8]));
                    let rtt = recv_wall.saturating_sub(sent_ts as u128);
                    latencies_us.push(rtt);
                }
                messages_received += 1;
            }
            _ => {
                // No response (solo client or timeout) — skip latency tracking but don't abort
            }
        }
    }

    // Graceful close
    let _ = write.send(Message::Close(None)).await;

    ClientResult {
        messages_sent,
        messages_received,
        latencies_us,
        error: None,
    }
}

#[tokio::main]
async fn main() {
    let cfg = parse_args();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  🚀 Kythia Relay Benchmark");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  URL:              {}", cfg.url);
    println!("  Clients:          {}", cfg.clients);
    println!("  Messages/client:  {}", cfg.messages_per_client);
    println!("  Payload size:     {} bytes", cfg.payload_size);
    println!("  Room:             {}", cfg.room_id);
    println!(
        "  Total messages:   {}",
        cfg.clients * cfg.messages_per_client
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let start = Instant::now();

    // Spawn all clients concurrently
    let mut handles = Vec::with_capacity(cfg.clients);
    for _ in 0..cfg.clients {
        let url = cfg.url.clone();
        let room_id = cfg.room_id.clone();
        let msgs = cfg.messages_per_client;
        let size = cfg.payload_size;
        handles.push(tokio::spawn(async move {
            run_client(url, room_id, msgs, size).await
        }));
    }

    // Collect results
    let mut all_latencies: Vec<u128> = Vec::new();
    let mut total_sent = 0usize;
    let mut total_received = 0usize;
    let mut errors = 0usize;

    for handle in handles {
        match handle.await {
            Ok(result) => {
                if result.error.is_some() {
                    errors += 1;
                    eprintln!("  ⚠️  Client error: {:?}", result.error);
                }
                total_sent += result.messages_sent;
                total_received += result.messages_received;
                all_latencies.extend(result.latencies_us);
            }
            Err(e) => {
                errors += 1;
                eprintln!("  ⚠️  Task panicked: {}", e);
            }
        }
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    // Sort latencies for percentiles
    all_latencies.sort_unstable();

    let throughput = total_sent as f64 / elapsed_secs;
    let avg_latency = if all_latencies.is_empty() {
        0
    } else {
        all_latencies.iter().sum::<u128>() / all_latencies.len() as u128
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  📊 Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Duration:         {:.2}s", elapsed_secs);
    println!("  Messages sent:    {}", total_sent);
    println!("  Messages received:{}", total_received);
    println!("  Errors:           {}", errors);
    println!("  Throughput:       {:.0} msg/s", throughput);
    println!(
        "  Data rate:        {:.2} MB/s",
        (total_sent * cfg.payload_size) as f64 / elapsed_secs / 1_000_000.0
    );

    if !all_latencies.is_empty() {
        println!("\n  ⏱️  Round-Trip Latency (µs)");
        println!("  ─────────────────────────────────────────");
        println!("  Min:    {}µs", all_latencies.first().unwrap());
        println!("  Avg:    {}µs", avg_latency);
        println!("  p50:    {}µs", percentile(&all_latencies, 50.0));
        println!("  p90:    {}µs", percentile(&all_latencies, 90.0));
        println!("  p95:    {}µs", percentile(&all_latencies, 95.0));
        println!("  p99:    {}µs", percentile(&all_latencies, 99.0));
        println!("  Max:    {}µs", all_latencies.last().unwrap());
    } else {
        println!("\n  ℹ️  No RTT data — need ≥2 clients in the same room for relay echo");
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if errors > 0 {
        std::process::exit(1);
    }
}
