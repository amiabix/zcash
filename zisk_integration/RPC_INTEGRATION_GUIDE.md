# 🚀 **RPC Integration Guide**

## 📋 **Overview**

This guide shows how to wire your ZisK-Zcash validator into an RPC interface for external clients.

## 🏗️ **Architecture**

```
┌─────────────────┐    HTTP/JSON    ┌─────────────────┐
│   External      │◄──────────────►│   RPC Service   │
│   Clients       │                │                 │
│  - Wallets      │                │  - HTTP Server  │
│  - L2 Rollups   │                │  - JSON API     │
│  - Sequencers   │                │  - Validation   │
└─────────────────┘                └─────────────────┘
                                           │
                                           ▼
┌─────────────────┐                ┌─────────────────┐
│   ZisK zkVM     │◄──────────────►│   Zcash Node    │
│                 │                │                 │
│  - main.rs      │                │  - gettxout     │
│  - validation   │                │  - getblock     │
│  - STARK proof  │                │  - gettxoutproof│
└─────────────────┘                └─────────────────┘
```

## 🔧 **RPC Endpoints**

### **1. Health Check**
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "memory_usage_bytes": 1048576,
  "processed_transactions": 1500,
  "generated_proofs": 750
}
```

### **2. Validate Single Transaction**
```http
POST /validate_transaction
```

**Request:**
```json
{
  "tx_bytes": "0x0400008085202f8901...",
  "utxos": [
    {
      "prev_txid": "0x1234567890abcdef...",
      "prev_index": 0,
      "value": 100000000,
      "script_pubkey": "0x76a914...",
      "merkle_proof": ["0xabc...", "0xdef..."],
      "leaf_index": 42,
      "height": 1000,
      "spendable": true
    }
  ],
  "prior_state_root": "0xabcdef1234567890...",
  "block_height": 1000,
  "consensus_branch_id": 230500
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "is_valid": true,
    "total_input_value": 100000000,
    "total_output_value": 99000000,
    "fee": 1000000,
    "transparent_balance": 1000000,
    "sapling_balance": 0,
    "orchard_balance": 0,
    "nullifiers_valid": true,
    "commitments_valid": true,
    "signatures_valid": true,
    "zk_proofs_valid": true,
    "new_state_root": "0xnewstate123...",
    "warnings": [],
    "errors": []
  },
  "error": null,
  "processing_time_us": 15000
}
```

### **3. Validate Batch**
```http
POST /validate_batch
```

**Request:**
```json
{
  "batch_bytes": "0x0400008085202f8901...",
  "utxos": [...],
  "prior_state_root": "0xabcdef1234567890...",
  "block_height": 1000,
  "consensus_branch_id": 230500
}
```

### **4. Generate Proof**
```http
POST /generate_proof
```

**Request:**
```json
{
  "validation_result": {
    "is_valid": true,
    "total_input_value": 100000000,
    "total_output_value": 99000000,
    "fee": 1000000,
    "new_state_root": "0xnewstate123...",
    "warnings": [],
    "errors": []
  },
  "tx_data": "0x0400008085202f8901...",
  "utxos": [...]
}
```

**Response:**
```json
{
  "success": true,
  "proof": {
    "proof_data": "0x1234567890abcdef...",
    "compressed_proof_data": "0xabcdef1234567890...",
    "public_inputs": [1, 2, 3, 4, 5],
    "metadata": {
      "proof_id": "proof_12345",
      "timestamp": 1640995200,
      "riscv_cycles": 2601,
      "memory_usage": 1048576,
      "proof_size": 250000,
      "compressed_size": 213000,
      "generation_time_us": 180000000,
      "verification_time_us": 7000
    }
  },
  "error": null,
  "generation_time_us": 180000000
}
```

### **5. Verify Proof**
```http
POST /verify_proof
```

**Request:**
```json
{
  "proof": {
    "proof_data": "0x1234567890abcdef...",
    "compressed_proof_data": "0xabcdef1234567890...",
    "public_inputs": [1, 2, 3, 4, 5],
    "metadata": {...}
  },
  "expected_public_inputs": [1, 2, 3, 4, 5]
}
```

## 🚀 **Usage Examples**

### **Start RPC Server**
```bash
# Set environment variables
export ZCASH_RPC_URL="http://localhost:8232"
export ZCASH_RPC_USER="user"
export ZCASH_RPC_PASSWORD="pass"
export SERVER_ADDR="0.0.0.0:8080"

# Start server
cargo run --example rpc_server --features rpc_server
```

### **Client Integration (Rust)**
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Validate transaction
    let response = client
        .post("http://localhost:8080/validate_transaction")
        .json(&json!({
            "tx_bytes": "0x0400008085202f8901...",
            "utxos": [...],
            "prior_state_root": "0xabcdef1234567890...",
            "block_height": 1000,
            "consensus_branch_id": 230500
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Validation result: {}", result);
    
    Ok(())
}
```

### **Client Integration (JavaScript)**
```javascript
const axios = require('axios');

async function validateTransaction(txBytes, utxos, priorStateRoot) {
    try {
        const response = await axios.post('http://localhost:8080/validate_transaction', {
            tx_bytes: txBytes,
            utxos: utxos,
            prior_state_root: priorStateRoot,
            block_height: 1000,
            consensus_branch_id: 230500
        });
        
        return response.data;
    } catch (error) {
        console.error('Validation failed:', error.response.data);
        throw error;
    }
}

// Usage
validateTransaction('0x0400008085202f8901...', utxos, '0xabcdef1234567890...')
    .then(result => console.log('Valid:', result.result.is_valid))
    .catch(error => console.error('Error:', error));
```

## 🔧 **Production Deployment**

### **Docker Configuration**
```dockerfile
FROM rust:1.70-slim

WORKDIR /app
COPY . .

RUN cargo build --release --features rpc_server

EXPOSE 8080

CMD ["./target/release/rpc_server"]
```

### **Environment Variables**
```bash
# Zcash node configuration
ZCASH_RPC_URL=http://zcash-node:8232
ZCASH_RPC_USER=production_user
ZCASH_RPC_PASSWORD=secure_password

# Server configuration
SERVER_ADDR=0.0.0.0:8080
RUST_LOG=info

# Optional: Use cookie authentication
ZCASH_COOKIE_FILE=/path/to/.zcash/.cookie
```

### **Load Balancing**
```yaml
# docker-compose.yml
version: '3.8'
services:
  zisk-validator-1:
    build: .
    environment:
      - SERVER_ADDR=0.0.0.0:8080
    ports:
      - "8081:8080"
  
  zisk-validator-2:
    build: .
    environment:
      - SERVER_ADDR=0.0.0.0:8080
    ports:
      - "8082:8080"
  
  nginx:
    image: nginx
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
```

## 🎯 **Ready for Production!**

Your RPC interface is now **production-ready** with:

- ✅ **Complete HTTP API** with JSON serialization
- ✅ **Health monitoring** and metrics
- ✅ **Error handling** and validation
- ✅ **Client examples** in multiple languages
- ✅ **Production deployment** configuration
- ✅ **Load balancing** support

**External clients can now submit transaction batches and receive validation results and STARK proofs!** 🚀
