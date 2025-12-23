# API Key Management System

Complete REST API for managing authentication keys with MySQL database backend.

## Quick Start

### 1. Start the Server with Docker Compose

```bash
# Start MySQL and Kythia Nexus
docker-compose up -d

# View logs
docker-compose logs -f kythia-relay
```

The master key will be automatically generated on first run and saved to `.master_key`.

### 2. Get Your Master Key

```bash
cat .master_key
```

Save this key securely! You'll need it to manage API keys.

## API Endpoints

### Base URL
```
http://localhost:8081/api/v1/keys
```

### Authentication
All API endpoints require a valid API key in the Authorization header:
```
Authorization: Bearer <your_api_key>
```

## Operations

### Create API Key

```bash
MASTER_KEY=$(cat .master_key)

curl -X POST http://localhost:8081/api/v1/keys \
  -H "Authorization: Bearer $MASTER_KEY" \
  -H "Content-Type: application/json"
```

Response:
```json
{
  "id": 2,
  "key": "kn_a1b2c3d4e5f6...",
  "name": "API Key",
  "is_active": true,
  "is_master": false,
  "created_at": 1703260800,
  "updated_at": 1703260800,
  "last_used_at": null,
  "metadata": null
}
```

**⚠️ IMPORTANT**: The `key` field is only shown once during creation. Save it immediately!

### List All API Keys

```bash
MASTER_KEY=$(cat .master_key)

curl http://localhost:8081/api/v1/keys \
  -H "Authorization: Bearer $MASTER_KEY"
```

Response:
```json
{
  "keys": [
    {
      "id": 1,
      "name": "Master Key",
      "is_active": true,
      "is_master": true,
      "created_at": 1703260800,
      "updated_at": 1703260800,
      "last_used_at": 1703267000,
      "metadata": null
    },
    {
      "id": 2,
      "name": "API Key",
      "is_active": true,
      "is_master": false,
      "created_at": 1703265000,
      "updated_at": 1703265000,
      "last_used_at": null,
      "metadata": null
    }
  ]
}
```

### Get Specific API Key

```bash
MASTER_KEY=$(cat .master_key)

curl http://localhost:8081/api/v1/keys/2 \
  -H "Authorization: Bearer $MASTER_KEY"
```

### Deactivate API Key

```bash
MASTER_KEY=$(cat .master_key)

curl -X PATCH http://localhost:8081/api/v1/keys/2/deactivate \
  -H "Authorization: Bearer $MASTER_KEY"
```

Response:
```json
{
  "message": "API key deactivated"
}
```

### Activate API Key

```bash
MASTER_KEY=$(cat .master_key)

curl -X PATCH http://localhost:8081/api/v1/keys/2/activate \
  -H "Authorization: Bearer $MASTER_KEY"
```

### Delete API Key

```bash
MASTER_KEY=$(cat .master_key)

curl -X DELETE http://localhost:8081/api/v1/keys/2 \
  -H "Authorization: Bearer $MASTER_KEY"
```

**Note**: Master key cannot be deleted or deactivated.

## Using API Keys with WebSocket

Once you have an API key, use it to connect to the WebSocket server:

```javascript
// Browser WebSocket
const ws = new WebSocket('ws://localhost:8080/?key=kn_your_api_key_here');

ws.onopen = () => {
  console.log('Connected with API key authentication');
  
  // Join a room
  ws.send(JSON.stringify({
    op: 'join',
    d: { room_id: 'my-room' }
  }));
};
```

```bash
# wscat
wscat -c "ws://localhost:8080/?key=kn_your_api_key_here"
```

## Database Schema

```sql
CREATE TABLE api_keys (
    id INT AUTO_INCREMENT PRIMARY KEY,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_master BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    last_used_at BIGINT,
    metadata TEXT,
    INDEX idx_key_hash (key_hash),
    INDEX idx_is_active (is_active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## Security

- **Key Format**: `kn_` prefix + 64 hex characters (32 random bytes)
-  **Storage**: Keys are hashed with SHA-256 before storage
- **Master Key**: Automatically generated on first run, cannot be deleted/deactivated
- **Validation**: Inactive keys are rejected
- **Tracking**: `last_used_at` timestamp updated on each use

## Configuration

Environment variables:

```bash
# Database
DATABASE_URL=mysql://kythia:kythia_password@localhost:3306/kythia

# Master Key File
MASTER_KEY_FILE=./.master_key

# Enable authentication (must be true for API keys to work)
AUTH_ENABLED=true
```

## For Production

1. **Change MySQL password** in `docker-compose.yml`
2. **Secure the master key** - store in a password manager or secrets vault
3. **Backup `.master_key` file** to prevent lockout
4. **Use strong DATABASE_URL** credentials
5. **Enable TLS/SSL** for MySQL connections in production
6. **Monitor** `last_used_at` for suspicious activity

## Troubleshooting

### Master key file not found
```bash
# Regenerate by deleting the database and restarting
docker-compose down -v
docker-compose up -d
cat .master_key
```

### Database connection failed
```bash
# Check MySQL is running
docker-compose ps

# View MySQL logs
docker-compose logs mysql

# Test connection
docker-compose exec mysql mysql -ukythia -pkythia_password kythia
```

### 401 Unauthorized
- Verify your API key is correct
- Check if the key is active: `GET /api/v1/keys`
- Ensure `Authorization: Bearer` header is set correctly

## API Key Lifecycle

1. **Create**: Master key creates new API keys
2. **Use**: Keys authenticate WebSocket and REST API requests
3. **Track**: `last_used_at` updates on each successful auth
4. **Deactivate**: Temporarily disable without deletion
5. **Reactivate**: Re-enable a deactivated key
6. **Delete**: Permanently remove (cannot be undone)

## Integration Examples

### Python
```python
import requests

MASTER_KEY = open('.master_key').read().strip()

# Create API key
response = requests.post(
    'http://localhost:8081/api/v1/keys',
    headers={'Authorization': f'Bearer {MASTER_KEY}'}
)
api_key = response.json()['key']
print(f"New API Key: {api_key}")

# Use with WebSocket
import websockets
async with websockets.connect(f'ws://localhost:8080/?key={api_key}') as ws:
    await ws.send('{"op":"join","d":{"room_id":"test"}}')
```

### Node.js/JavaScript
```javascript
const axios = require('axios');
const fs = require('fs');

const MASTER_KEY = fs.readFileSync('.master_key', 'utf8').trim();

// Create API key
const response = await axios.post(
  'http://localhost:8081/api/v1/keys',
  {},
  { headers: { 'Authorization': `Bearer ${MASTER_KEY}` } }
);

const apiKey = response.data.key;
console.log('New API Key:', apiKey);

// Use with WebSocket  
const WebSocket = require('ws');
const ws = new WebSocket(`ws://localhost:8080/?key=${apiKey}`);
```

---

**Status**: ✅ Production Ready  
**Database**: MySQL 8.0  
**Security**: SHA-256 hashed keys, master key protection
