<div align="center">
  <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Objects/Rocket.png" alt="Rocket" width="120" height="120" />
  <h1>Kythia Nexus Core</h1>
  <p><strong>A High-Performance WebSocket Signaling Server built with Rust</strong></p>

  <p>
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
    <img src="https://img.shields.io/badge/Tokio-000000?style=for-the-badge&logo=tokio&logoColor=white" alt="Tokio" />
    <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License" />
  </p>
</div>

---

## 🌟 Overview

**Kythia Nexus Core** is the high-performance backbone of the Kythia ecosystem. It serves as a lightweight, robust, and extremely fast WebSocket signaling server designed to handle real-time data broadcasting, specifically optimized for audio/video streaming and "wormhole" functionality.

Built with **Rust** and powered by **Tokio**, it ensures minimal latency and maximum throughput while maintaining memory safety and stability.

## ✨ Features

- 🚀 **Blazing Fast**: Leverages Rust's performance and Tokio's asynchronous runtime.
- 📡 **WebSocket Protocol**: Real-time communication using `tokio-tungstenite`.
- 🏠 **Room-Based Signaling**: Simple and efficient room management for peer-to-peer or broadcast scenarios.
- 🛡️ **Robust & Reliable**: Implements bounded channels and `try_send` to prevent "Slow Consumer" problems and OOM crashes.
- ⚙️ **Configurable**: Easy setup via environment variables (`.env` support).
- 📊 **Thread-Safe State**: Uses `DashMap` for high-concurrency room and peer management.

## 🛠️ Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/) (Edition 2024)
- **Runtime**: [Tokio](https://tokio.rs/)
- **WebSockets**: [Tungstenite](https://github.com/snapview/tungstenite-rs)
- **Serialization**: [Serde](https://serde.rs/)
- **Configuration**: [Dotenvy](https://github.com/allan2/dotenvy)
- **Logging**: [Env Logger](https://github.com/rust-cli/env_logger) & [Log](https://github.com/rust-lang/log)

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/kythia-relay.git
   cd kythia-relay
   ```

2. **Setup environment variables**:
   ```bash
   cp .env.example .env
   ```
   Edit `.env` to customize your `HOST` and `PORT`.

3. **Build and Run**:
   ```bash
   cargo run --release
   ```

## ⚙️ Configuration

The server can be configured using environment variables or a `.env` file:

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | The address to bind the server to | `0.0.0.0` |
| `PORT` | The port to listen on | `8080` |
| `RUST_LOG` | Logging level (`info`, `debug`, `error`) | `info` |

## 📡 Protocol

### Join a Room
Send a JSON message to join a specific room:
```json
{
  "op": "join",
  "d": {
    "room_id": "my-awesome-room"
  }
}
```

### Broadcasting Data
Once in a room, any **Binary** message sent by a client will be broadcasted to all other clients in the same room.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <p>Built with ❤️ by the Kythia Team</p>
</div>
