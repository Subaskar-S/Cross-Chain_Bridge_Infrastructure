# API Reference

This document provides comprehensive documentation for the Cross-Chain Bridge API.

## Base URL

- **Development**: `http://localhost:3001`
- **Staging**: `https://bridge-api-staging.yourdomain.com`
- **Production**: `https://bridge-api.yourdomain.com`

## Authentication

Currently, the API does not require authentication for read-only endpoints. Write operations (if any) would require proper authentication in a production environment.

## Rate Limiting

- **Default**: 100 requests per minute per IP
- **Burst**: Up to 200 requests in a 10-second window
- **Headers**: Rate limit information is included in response headers

## Response Format

All API responses follow a consistent JSON format:

```json
{
  "data": { ... },
  "status": "success|error",
  "message": "Human readable message",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Health and Status Endpoints

### GET /health

Returns the health status of the bridge service.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "bridge_stats": {
    "ethereum_processed_txs": 100,
    "polkadot_processed_txs": 95,
    "pending_signatures": 2,
    "active_validators": 3
  }
}
```

### GET /status

Returns detailed bridge status information.

**Response:**
```json
{
  "status": "operational",
  "ethereum_block": 12345,
  "polkadot_block": 6789,
  "validators": [
    {
      "id": "validator_0",
      "address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "active": true,
      "stake": "1000",
      "uptime": 99.5
    }
  ],
  "recent_transactions": [
    {
      "tx_hash": "0x1234567890abcdef",
      "chain": "ethereum",
      "status": "confirmed",
      "amount": "1000",
      "token": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
      "user": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "block_number": 12345,
      "timestamp": "2024-01-01T12:00:00Z"
    }
  ]
}
```

### GET /stats

Returns bridge statistics.

**Response:**
```json
{
  "ethereum_processed_txs": 100,
  "polkadot_processed_txs": 95,
  "pending_signatures": 2,
  "active_validators": 3
}
```

## Transaction Endpoints

### GET /transactions

Returns a list of bridge transactions with pagination and filtering.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 20, max: 100)
- `chain` (optional): Filter by chain ("ethereum" or "polkadot")
- `status` (optional): Filter by status ("pending", "confirmed", "failed")
- `user` (optional): Filter by user address
- `token` (optional): Filter by token address
- `from_block` (optional): Filter from block number
- `to_block` (optional): Filter to block number

**Example Request:**
```
GET /transactions?page=1&limit=10&chain=ethereum&status=confirmed
```

**Response:**
```json
[
  {
    "tx_hash": "0x1234567890abcdef",
    "chain": "ethereum",
    "status": "confirmed",
    "amount": "1000",
    "token": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
    "user": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "block_number": 12345,
    "timestamp": "2024-01-01T12:00:00Z"
  }
]
```

### GET /transactions/:tx_hash

Returns details for a specific transaction.

**Parameters:**
- `tx_hash`: Transaction hash

**Response:**
```json
{
  "tx_hash": "0x1234567890abcdef",
  "chain": "ethereum",
  "status": "confirmed",
  "amount": "1000",
  "token": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
  "user": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  "block_number": 12345,
  "timestamp": "2024-01-01T12:00:00Z",
  "confirmations": 12,
  "signatures": [
    {
      "validator": "validator_0",
      "signature": "0x...",
      "timestamp": "2024-01-01T12:01:00Z"
    }
  ]
}
```

## Validator Endpoints

### GET /validators

Returns a list of bridge validators.

**Response:**
```json
[
  {
    "id": "validator_0",
    "address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "active": true,
    "stake": "1000",
    "uptime": 99.5
  }
]
```

### GET /validators/:validator_id

Returns details for a specific validator.

**Parameters:**
- `validator_id`: Validator identifier

**Response:**
```json
{
  "id": "validator_0",
  "address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
  "active": true,
  "stake": "1000",
  "uptime": 99.5,
  "last_seen": "2024-01-01T12:00:00Z",
  "signatures_count": 150,
  "missed_signatures": 2
}
```

## Token Endpoints

### GET /tokens

Returns a list of supported tokens.

**Response:**
```json
[
  {
    "ethereum_address": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
    "polkadot_asset_id": 1,
    "name": "Test Token",
    "symbol": "TEST",
    "decimals": 18,
    "total_locked": "10000",
    "total_minted": "9500"
  }
]
```

### GET /tokens/:token_address

Returns details for a specific token.

**Parameters:**
- `token_address`: Ethereum token contract address

**Response:**
```json
{
  "ethereum_address": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
  "polkadot_asset_id": 1,
  "name": "Test Token",
  "symbol": "TEST",
  "decimals": 18,
  "total_locked": "10000",
  "total_minted": "9500",
  "recent_transfers": [
    {
      "tx_hash": "0x...",
      "amount": "100",
      "direction": "ethereum_to_polkadot",
      "timestamp": "2024-01-01T12:00:00Z"
    }
  ]
}
```

## Block Information Endpoints

### GET /blocks/ethereum/latest

Returns the latest Ethereum block number being monitored.

**Response:**
```json
{
  "block_number": 12345,
  "timestamp": "2024-01-01T12:00:00Z",
  "confirmations_required": 12
}
```

### GET /blocks/polkadot/latest

Returns the latest Polkadot block number being monitored.

**Response:**
```json
{
  "block_number": 6789,
  "timestamp": "2024-01-01T12:00:00Z",
  "confirmations_required": 6
}
```

## Event Endpoints

### GET /events

Returns a list of bridge events.

**Query Parameters:**
- `page` (optional): Page number
- `limit` (optional): Items per page
- `chain` (optional): Filter by chain
- `event_type` (optional): Filter by event type

**Response:**
```json
[
  {
    "id": "event_123",
    "type": "bridge_lock",
    "chain": "ethereum",
    "tx_hash": "0x...",
    "block_number": 12345,
    "timestamp": "2024-01-01T12:00:00Z",
    "data": {
      "user": "0x...",
      "token": "0x...",
      "amount": "1000"
    }
  }
]
```

## Metrics Endpoint

### GET /metrics

Returns Prometheus-formatted metrics.

**Response:**
```
# HELP bridge_processed_transactions_total Total number of processed transactions
# TYPE bridge_processed_transactions_total counter
bridge_processed_transactions_total{chain="ethereum"} 100
bridge_processed_transactions_total{chain="polkadot"} 95

# HELP bridge_active_validators Number of active validators
# TYPE bridge_active_validators gauge
bridge_active_validators 3

# HELP bridge_pending_signatures Number of pending signatures
# TYPE bridge_pending_signatures gauge
bridge_pending_signatures 2
```

## WebSocket API

### Connection

Connect to the WebSocket endpoint for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');
```

### Message Types

#### Bridge Events
```json
{
  "type": "bridge_event",
  "event_type": "ethereum_lock",
  "data": {
    "user": "0x...",
    "token": "0x...",
    "amount": "1000",
    "tx_hash": "0x..."
  }
}
```

#### Statistics Updates
```json
{
  "type": "stats_update",
  "stats": {
    "ethereum_processed_txs": 101,
    "polkadot_processed_txs": 95,
    "pending_signatures": 1,
    "active_validators": 3
  }
}
```

#### Validator Updates
```json
{
  "type": "validator_update",
  "validator": {
    "id": "validator_0",
    "active": true,
    "uptime": 99.6
  }
}
```

## Error Responses

### Error Format
```json
{
  "error": "Error Type",
  "message": "Detailed error message",
  "code": 400,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Common Error Codes

- **400 Bad Request**: Invalid request parameters
- **404 Not Found**: Resource not found
- **429 Too Many Requests**: Rate limit exceeded
- **500 Internal Server Error**: Server error
- **503 Service Unavailable**: Bridge temporarily unavailable

## SDK Examples

### JavaScript/TypeScript

```typescript
import axios from 'axios';

const bridgeApi = axios.create({
  baseURL: 'https://bridge-api.yourdomain.com',
  timeout: 10000,
});

// Get bridge status
const status = await bridgeApi.get('/status');
console.log(status.data);

// Get transaction details
const tx = await bridgeApi.get('/transactions/0x...');
console.log(tx.data);

// WebSocket connection
const ws = new WebSocket('wss://bridge-api.yourdomain.com/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Bridge event:', data);
};
```

### Python

```python
import requests
import websocket
import json

# REST API
response = requests.get('https://bridge-api.yourdomain.com/status')
status = response.json()
print(status)

# WebSocket
def on_message(ws, message):
    data = json.loads(message)
    print(f"Bridge event: {data}")

ws = websocket.WebSocketApp(
    "wss://bridge-api.yourdomain.com/ws",
    on_message=on_message
)
ws.run_forever()
```

This API reference provides comprehensive documentation for integrating with the Cross-Chain Bridge API. For additional support or questions, please refer to the project documentation or open an issue on GitHub.
