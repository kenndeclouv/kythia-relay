# 🚀 Kythia RelayCore Setup Guide

> **Beginner-Friendly Guide** - This guide will walk you through setting up Kythia RelayCore from scratch with detailed explanations for each step.

## 📋 Table of Contents

- [What is Kythia RelayCore?](#what-is-kythia-relaycore)
- [Prerequisites](#prerequisites)
- [Quick Start with Docker (Recommended)](#quick-start-with-docker-recommended)
- [Manual Installation](#manual-installation)
- [Configuration Guide](#configuration-guide)
- [Testing Your Setup](#testing-your-setup)
- [Common Issues \u0026 Troubleshooting](#common-issues--troubleshooting)
- [Next Steps](#next-steps)

---

## What is Kythia RelayCore?

Kythia RelayCore is a **high-performance WebSocket server** written in Rust that enables real-time communication between multiple clients. Think of it as a "relay station" that broadcasts messages between connected clients in different "rooms."

**Use Cases:**
- Real-time audio/video streaming relay
- Live chat applications
- Multiplayer game signaling
- IoT device communication
- Collaborative tools (whiteboarding, document editing)

---

## Prerequisites

Before you begin, make sure you have the following installed on your system:

### Required:

1. **Docker \u0026 Docker Compose** (Easiest method)
   - **Windows/Mac**: [Docker Desktop](https://www.docker.com/products/docker-desktop)
   - **Linux**: 
     ```bash
     # Ubuntu/Debian
     sudo apt update
     sudo apt install docker.io docker-compose
     
     # Fedora
     sudo dnf install docker docker-compose
     ```

   **Verify Installation:**
   ```bash
   docker --version
   docker-compose --version
   ```

### OR (For manual installation):

2. **Rust Programming Language**
   ```bash
   # Install Rust via rustup (official installer)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Follow the prompts, then restart your terminal
   # Verify installation:
   rustc --version
   cargo --version
   ```

3. **MySQL 8.0+**
   - **Windows**: [MySQL Installer](https://dev.mysql.com/downloads/installer/)
   - **Mac**: `brew install mysql`
   - **Linux**: 
     ```bash
     sudo apt install mysql-server  # Ubuntu/Debian
     sudo dnf install mysql-server  # Fedora
     ```

4. **Git**
   ```bash
   git --version  # Check if already installed
   ```
   If not installed: [git-scm.com/downloads](https://git-scm.com/downloads)

---

## Quick Start with Docker (Recommended)

This is the **easiest and fastest** way to get started!

### Step 1: Clone the Repository

```bash
# Clone the project
git clone https://github.com/your-username/kythia-relay.git

# Navigate into the project directory
cd kythia-relay
```

### Step 2: Review Configuration

The project includes a Docker Compose configuration that sets up everything automatically:
- MySQL database
- Kythia RelayCore server
- Networking between services

**Optional:** Review the configuration:
```bash
cat docker-compose.yml
```

### Step 3: Start the Services

```bash
# Start both MySQL and Kythia RelayCore in the background
docker-compose up -d

# View logs to see what's happening
docker-compose logs -f
```

**What's happening:**
1. MySQL database starts first
2. Once MySQL is healthy, Kythia RelayCore starts
3. On first run, a **Master API Key** is automatically generated
4. The key is saved to `.master_key` file

### Step 4: Get Your Master API Key

```bash
# Display your master key
cat .master_key
```

**Output example:**
```
kythia-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8c9d0e1f2
```

> ⚠️ **IMPORTANT**: Save this key securely! You'll need it to manage API keys and authenticate connections.

### Step 5: Verify Everything is Running

```bash
# Check service status
docker-compose ps

# Test health endpoint
curl http://localhost:8081/health
```

**Expected response:**
```json
{
  "status": "healthy"
}
```

**🎉 Congratulations!** Your server is now running at:
- **WebSocket**: `ws://localhost:8080`
- **HTTP API**: `http://localhost:8081`

---

## Manual Installation

If you prefer to run without Docker or need a custom setup:

### Step 1: Clone the Repository

```bash
git clone https://github.com/your-username/kythia-relay.git
cd kythia-relay
```

### Step 2: Set Up MySQL Database

```bash
# Start MySQL (if not already running)
sudo systemctl start mysql  # Linux
# Or on Mac:
brew services start mysql

# Create database and user
mysql -u root -p
```

Run these SQL commands:
```sql
CREATE DATABASE kythia;
CREATE USER 'kythia'@'localhost' IDENTIFIED BY 'kythia_password';
GRANT ALL PRIVILEGES ON kythia.* TO 'kythia'@'localhost';
FLUSH PRIVILEGES;
EXIT;
```

### Step 3: Configure Environment Variables

```bash
# Copy example configuration
cp .env.example .env

# Edit configuration
nano .env  # or use your preferred editor
```

**Minimum configuration for `.env`:**
```bash
# Server Settings
HOST=0.0.0.0
PORT=8080
HTTP_PORT=8081

# Database Settings
DATABASE_URL=mysql://kythia:kythia_password@localhost:3306/kythia

# Security
AUTH_ENABLED=true
MASTER_KEY_FILE=./.master_key

# Logging
RUST_LOG=info
```

### Step 4: Build the Project

```bash
# Download dependencies and build
cargo build --release

# This may take 5-10 minutes on first build
```

### Step 5: Run the Server

```bash
# Run in release mode for best performance
./target/release/kythia-relay

# Or run in development mode with auto-reload
cargo run
```

**Expected output:**
```
🔌 Connecting to database...
✅ Database connected successfully
🔑 NEW MASTER KEY GENERATED!
   Master Key: kythia-...
   Saving to: ./.master_key
🚀 Kythia RelayCore listening on: 0.0.0.0:8080
📊 Metrics available at: http://localhost:8081/metrics
💚 Health check at: http://localhost:8081/health
🔐 Authentication: ENABLED
```

---

## Configuration Guide

### Understanding Environment Variables

| Variable | What it does | Recommended Value |
|----------|-------------|-------------------|
| `HOST` | IP address the server listens on | `0.0.0.0` (all interfaces) |
| `PORT` | WebSocket connection port | `8080` |
| `HTTP_PORT` | HTTP API/metrics port | `8081` |
| `DATABASE_URL` | MySQL connection string | `mysql://user:pass@host:port/db` |
| `AUTH_ENABLED` | Require API keys to connect | `true` (recommended) |
| `METRICS_ENABLED` | Enable monitoring endpoints | `true` |
| `RUST_LOG` | Log verbosity level | `info` or `debug` |

### Performance Tuning

```bash
# .env file

# How many messages can queue up per client before dropping
CHANNEL_BUFFER_SIZE=500

# Maximum clients per room (0 = unlimited)
MAX_ROOM_SIZE=0

# Maximum size of a single message (in bytes)
MAX_MESSAGE_SIZE=1048576  # 1MB

# How long before idle connections timeout (seconds)
CONNECTION_TIMEOUT=60

# Messages per second allowed per client
RATE_LIMIT_PER_SECOND=100
```

**When to adjust these:**
- **High traffic**: Increase `CHANNEL_BUFFER_SIZE` to 1000+
- **Voice/Video**: Increase `MAX_MESSAGE_SIZE` to 5-10MB
- **Strict control**: Lower `MAX_ROOM_SIZE` to limit room participants

### Security Configuration

```bash
# Enable authentication (highly recommended for production)
AUTH_ENABLED=true

# If AUTH_ENABLED=true, you don't need to set AUTH_SECRET
# The system uses API keys from the database instead
```

---

## Testing Your Setup

### Test 1: Health Check

```bash
curl http://localhost:8081/health
```

**Expected:** `{"status":"healthy"}`

### Test 2: View Metrics

```bash
curl http://localhost:8081/metrics
```

**Expected:**
```json
{
  "total_connections": 0,
  "active_connections": 0,
  "total_rooms_created": 0,
  "active_rooms": 0,
  "messages_sent": 0,
  "messages_received": 0,
  "messages_dropped": 0,
  "bytes_sent": 0,
  "bytes_received": 0
}
```

### Test 3: Create an API Key

```bash
# Save your master key to a variable
MASTER_KEY=$(cat .master_key)

# Create a new API key
curl -X POST http://localhost:8081/api/v1/keys \
  -H "Authorization: Bearer $MASTER_KEY" \
  -H "Content-Type: application/json"
```

**Expected:** JSON response with your new API key
```json
{
  "id": 2,
  "key": "kythia-...",
  "name": "API Key",
  "is_active": true,
  ...
}
```

> ⚠️ **Save the `key` value immediately! It's only shown once.**

### Test 4: Connect via WebSocket

**Option A: Using wscat (Node.js tool)**

```bash
# Install wscat globally
npm install -g wscat

# Replace YOUR_API_KEY with the key from Test 3
wscat -c "ws://localhost:8080/?key=YOUR_API_KEY"

# Once connected, join a room:
\u003e {"op":"join","d":{"room_id":"test-room"}}

# Send a ping:
\u003e {"op":"ping"}
```

**Option B: Using a JavaScript client**

Create a file `test.html`:

```html
\u003c!DOCTYPE html\u003e
\u003chtml\u003e
\u003chead\u003e\u003ctitle\u003eKythia Test\u003c/title\u003e\u003c/head\u003e
\u003cbody\u003e
  \u003ch1\u003eKythia RelayCore Test\u003c/h1\u003e
  \u003cdiv id="status"\u003eConnecting...\u003c/div\u003e
  
  \u003cscript\u003e
    const apiKey = 'YOUR_API_KEY'; // Replace with your key
    const ws = new WebSocket(`ws://localhost:8080/?key=${apiKey}`);
    
    ws.onopen = () =\u003e {
      document.getElementById('status').innerText = '✅ Connected!';
      
      // Join a room
      ws.send(JSON.stringify({
        op: 'join',
        d: { room_id: 'test-room' }
      }));
      
      console.log('Joined test-room');
    };
    
    ws.onmessage = (event) =\u003e {
      console.log('Received:', event.data);
    };
    
    ws.onerror = (error) =\u003e {
      console.error('WebSocket error:', error);
      document.getElementById('status').innerText = '❌ Connection error';
    };
  \u003c/script\u003e
\u003c/body\u003e
\u003c/html\u003e
```

Open `test.html` in your browser and check the console.

---

## Common Issues \u0026 Troubleshooting

### ❌ Error: "Failed to bind to 0.0.0.0:8080"

**Cause:** Port 8080 is already in use by another application.

**Solution:**
```bash
# Find what's using the port
sudo lsof -i :8080  # Linux/Mac
netstat -ano | findstr :8080  # Windows

# Either stop that application, or change PORT in .env
PORT=8090  # Use a different port
```

### ❌ Error: "Failed to connect to database"

**Cause:** MySQL is not running or credentials are incorrect.

**Solution:**
```bash
# Check if MySQL is running
sudo systemctl status mysql  # Linux
brew services list | grep mysql  # Mac

# Start MySQL if stopped
sudo systemctl start mysql  # Linux
brew services start mysql  # Mac

# Test connection manually
mysql -u kythia -pkythia_password kythia
```

If connection still fails, verify `DATABASE_URL` in `.env` matches your MySQL setup.

### ❌ Error: "Master key file not found"

**Cause:** The `.master_key` file was deleted or moved.

**Solution:**
```bash
# Option 1: Regenerate by clearing database
docker-compose down -v  # Removes database volumes
docker-compose up -d
cat .master_key

# Option 2: Manually access master key from database
mysql -u kythia -pkythia_password kythia
SELECT * FROM api_keys WHERE is_master = TRUE;
# The key_hash is stored, but original key is not recoverable
# You'll need to regenerate
```

### ❌ WebSocket Connection: "401 Unauthorized"

**Cause:** Invalid or missing API key.

**Solution:**
1. Verify `AUTH_ENABLED=true` in your `.env`
2. Check your API key is correct
3. Verify the key is active:
   ```bash
   curl http://localhost:8081/api/v1/keys \
     -H "Authorization: Bearer $(cat .master_key)"
   ```
4. Make sure you're including `?key=YOUR_API_KEY` in the WebSocket URL

### ❌ Docker: "Port is already allocated"

**Cause:** Ports 8080 or 8081 are in use on your host.

**Solution:**
Edit `docker-compose.yml` to use different ports:
```yaml
ports:
  - "9080:8080"  # Change host port (left side)
  - "9081:8081"
```

Then connect to `ws://localhost:9080` instead.

### 🐌 Performance: High memory usage

**Cause:** Too many buffered messages or large room sizes.

**Solution:**
```bash
# Reduce buffer size in .env
CHANNEL_BUFFER_SIZE=200
MAX_ROOM_SIZE=50
MAX_MESSAGE_SIZE=524288  # 512KB instead of 1MB
```

---

## Next Steps

### 🎓 Learn the Protocol

Read the [API_KEYS.md](API_KEYS.md) guide to understand:
- How to create and manage API keys
- WebSocket protocol operations (`join`, `leave`, `ping`, etc.)
- How to send and receive messages

### 📚 Explore Advanced Configuration

Check out [ARCHITECTURE.md](ARCHITECTURE.md) for:
- Detailed system architecture
- Performance optimization techniques
- Production deployment best practices
- Security hardening

### 🔧 Build an Application

Try building a simple application:
1. **Chat Room**: Connect multiple clients to the same room
2. **Audio Relay**: Stream audio between browsers
3. **Live Dashboard**: Show real-time metrics from `/metrics` endpoint

### 🌐 Deploy to Production

When you're ready to deploy:
1. Use WSS (Secure WebSocket) with a reverse proxy (nginx, Caddy)
2. Change all default passwords
3. Set strong `MYSQL_ROOT_PASSWORD` in `docker-compose.yml`
4. Back up your `.master_key` file
5. Enable firewall rules
6. Monitor logs and metrics regularly

### 📖 Read Full Documentation

- **README.md**: Feature overview and quick reference
- **API_KEYS.md**: Complete API key management guide
- **ARCHITECTURE.md**: Deep technical documentation

---

## 🆘 Need Help?

- **Issues**: [GitHub Issues](https://github.com/your-username/kythia-relay/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/kythia-relay/discussions)
- **Documentation**: Check the `docs/` folder for more guides

---

\u003cdiv align="center"\u003e
  \u003cp\u003e✨ \u003cstrong\u003eYou're all set up!\u003c/strong\u003e ✨\u003c/p\u003e
  \u003cp\u003eNow go build something amazing with Kythia RelayCore! 🚀\u003c/p\u003e
\u003c/div\u003e
