## 🌟 Overview

**Kythia RelayCore** is an enterprise-grade, production-ready WebSocket signaling server that forms the backbone of the Kythia ecosystem. It delivers exceptional performance, reliability, and scalability for real-time data broadcasting, optimized for audio/video streaming and multi-room communication.

Built with **Rust** and powered by **Tokio**, it ensures minimal latency, maximum throughput, and memory safety while handling thousands of concurrent connections.

## ✨ Features

### Core Functionality
- 🚀 **Blazing Fast**: Leverages Rust's zero-cost abstractions and Tokio's asynchronous runtime
- 📡 **WebSocket Protocol**: Full-duplex real-time communication using `tokio-tungstenite`
- 🏠 **Room-Based Signaling**: Efficient multi-room management for peer-to-peer or broadcast scenarios
- 🛡️ **Robust & Reliable**: Implements bounded channels and `try_send` to prevent "Slow Consumer" problems and OOM crashes
- ⚙️ **Highly Configurable**: Comprehensive environment variable configuration
- 📊 **Thread-Safe State**: Uses `DashMap` for high-concurrency room and peer management

### Security & Protection
- 🔐 **Authentication Support**: Optional JWT-based authentication system
- 🚦 **Rate Limiting**: Per-client rate limiting using token bucket algorithm
- 📏 **Message Size Validation**: Configurable maximum message size limits
- 👤 **Non-Root Execution**: Docker image runs as non-root user

### Monitoring & Observability
- 📊 **Metrics Collection**: Real-time metrics via HTTP endpoint
- 💚 **Health Checks**: Dedicated health check endpoint for orchestration
- 📈 **Performance Tracking**: Track connections, rooms, messages, and bandwidth
- 🔍 **Structured Logging**: Comprehensive logging with configurable levels

### Protocol Extensions
- 📥 **Join/Leave Operations**: Explicit room management
- 🏓 **Application-Level Ping/Pong**: Custom heartbeat mechanism
- 👥 **List Clients**: Query active participants in a room
- 🗺️ **List Rooms**: Admin endpoint to view all active rooms
- ⚠️ **Error Responses**: Structured error messages with codes

### Deployment & DevOps
- 🐳 **Docker Support**: Multi-stage optimized Dockerfile
- 📦 **Docker Compose**: Ready-to-use compose configuration
- 🎯 **Graceful Shutdown**: SIGTERM/SIGINT handling with connection draining
- ☁️ **Cloud-Ready**: Kubernetes manifests available

## 🛠️ Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/) (Edition 2024)
- **Runtime**: [Tokio](https://tokio.rs/)
- **WebSockets**: [Tungstenite](https://github.com/snapview/tungstenite-rs)
- **Serialization**: [Serde](https://serde.rs/)
- **Configuration**: [Dotenvy](https://github.com/allan2/dotenvy)
- **Authentication**: [JSON Web Tokens](https://github.com/Keats/jsonwebtoken)
- **Rate Limiting**: [Governor](https://github.com/boinkor-net/governor)
- **State Management**: [DashMap](https://github.com/xacrimon/dashmap)

## 📚 Documentation

### For Beginners
**[📖 Setup Guide (SETUP.md)](SETUP.md)** - Start here if you're new!
- Step-by-step installation instructions
- Configuration walkthrough with explanations
- Testing your setup with examples
- Common issues and troubleshooting
- Quick start with Docker

### For Advanced Users
**[🏗️ Architecture Guide (ARCHITECTURE.md)](ARCHITECTURE.md)** - Deep technical documentation
- Complete system architecture and design
- Component breakdown and data flow
- WebSocket protocol specification
- REST API reference
- Performance optimization techniques
- Production deployment strategies
- Security implementation details
- Monitoring and observability

### API Management
**[🔑 API Keys Guide (API_KEYS.md)](API_KEYS.md)** - Authentication system
- Creating and managing API keys
- REST API endpoints
- Integration examples
- Security best practices

## 🚀 Getting Started

> **New to Kythia RelayCore?** Check out our comprehensive **[Setup Guide (SETUP.md)](SETUP.md)** for detailed installation instructions!

### Quick Start with Docker

```bash
# Clone and start
git clone https://github.com/kenndeclouv/kythia-relay.git
cd kythia-relay
docker-compose up -d

# Get your master API key
cat .master_key
```

Your server is now running at `ws://localhost:8080` and `http://localhost:8081`!

### Manual Installation

For manual installation, advanced configuration, and troubleshooting, see **[SETUP.md](SETUP.md)**.

## ⚙️ Configuration

The server is configured using environment variables. Copy `.env.example` to `.env` and customize:

### Essential Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | WebSocket port | `8080` |
| `HTTP_PORT` | HTTP API/metrics port | `8081` |
| `DATABASE_URL` | MySQL connection string | See `.env.example` |
| `AUTH_ENABLED` | Enable API key authentication | `false` |
| `RUST_LOG` | Logging level | `info` |

**For complete configuration options and performance tuning**, see **[SETUP.md - Configuration Guide](SETUP.md#configuration-guide)** and **[ARCHITECTURE.md - Performance Architecture](ARCHITECTURE.md#performance-architecture)**.

## 📡 WebSocket Protocol

### Connection

Connect to the WebSocket server:
```
ws://localhost:8080/
```

With authentication (if enabled):
```
ws://localhost:8080/?token=YOUR_JWT_TOKEN
```

### Protocol Messages

All messages are JSON-formatted with an `op` (operation) field and optional `d` (data) field.

#### Join a Room
```json
{
  "op": "join",
  "d": {
    "room_id": "my-awesome-room"
  }
}
```

#### Leave Current Room
```json
{
  "op": "leave"
}
```

#### Application-Level Ping
```json
{
  "op": "ping"
}
```

Server responds with:
```json
{
  "op": "pong"
}
```

#### List Clients in Room
```json
{
  "op": "list_clients"
}
```

Server responds with:
```json
{
  "op": "response",
  "d": {
    "Clients": [
      {
        "id": "127.0.0.1:12345",
        "joined_at": 1703260800
      }
    ]
  }
}
```

#### List All Rooms (Admin)
```json
{
  "op": "list_rooms"
}
```

Server responds with:
```json
{
  "op": "response",
  "d": {
    "Rooms": [
      {
        "room_id": "room-1",
        "client_count": 5,
        "created_at": 1703260800,
        "is_private": false
      }
    ]
  }
}
```

### Binary Messages

Once in a room, any **Binary** message sent by a client will be broadcasted to all other clients in the same room (excluding the sender).

### Error Responses

```json
{
  "op": "error",
  "d": {
    "message": "Not in any room",
    "code": "NOT_IN_ROOM"
  }
}
```

## 📊 HTTP Endpoints

### Health Check

```bash
curl http://localhost:8081/health
```

Response:
```json
{
  "status": "healthy"
}
```

### Metrics

```bash
curl http://localhost:8081/metrics
```

Response:
```json
{
  "total_connections": 1250,
  "active_connections": 42,
  "total_rooms_created": 150,
  "active_rooms": 8,
  "messages_sent": 50000,
  "messages_received": 50000,
  "messages_dropped": 12,
  "bytes_sent": 52428800,
  "bytes_received": 52428800
}
```

## 🧪 Testing

### Manual Testing with wscat

```bash
# Install wscat
npm install -g wscat

# Connect to server
wscat -c ws://localhost:8080

# Send join message
> {"op":"join","d":{"room_id":"test-room"}}

# Send ping
> {"op":"ping"}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

This project is licensed under the CC BY-NC 4.0 License. See the [LICENSE](LICENSE) file for details.
Copyright © 2025 Kythia Labs - All rights reserved.

## 🏗️ Architecture

- **Multi-threaded**: Each WebSocket connection runs in its own Tokio task
- **Lock-free**: Uses DashMap for concurrent access without traditional locks
- **Zero-copy**: Binary messages use `Arc` for efficient memory sharing
- **Bounded channels**: Prevents memory exhaustion from slow consumers

## 🔒 Security Best Practices

- Always use strong `AUTH_SECRET` (min 32 characters) in production
- Enable rate limiting to prevent abuse
- Set appropriate `MAX_MESSAGE_SIZE` and `MAX_ROOM_SIZE` limits
- Use HTTPS/WSS in production with a reverse proxy (nginx, Caddy)
- Monitor metrics regularly for anomalies

## 📈 Performance Tips

- Adjust `CHANNEL_BUFFER_SIZE` based on your workload
- Use `RUST_LOG=error` in production for better performance
- Deploy behind a load balancer for horizontal scaling
- Use connection pooling on client side
- Monitor `messages_dropped` metric and adjust rate limits

---

<div align="center">
  <p>Built with ❤️ by the Kythia Labs</p>
  <p>⭐ Star us on GitHub if you find this useful!</p>
</div>
