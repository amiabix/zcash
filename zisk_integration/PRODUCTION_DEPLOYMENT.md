# 🚀 **Production Deployment Guide**

## 📋 **Overview**

This guide covers deploying the ZisK-Zcash validator in production with real Zcash node integration.

## 🏗️ **Architecture**

```
┌─────────────────┐    HTTP/RPC    ┌─────────────────┐
│   Host App      │◄──────────────►│   zcashd Node   │
│                 │                │                 │
│  - NodeBridge   │                │  - gettxout     │
│  - Serializer   │                │  - getblock     │
│  - Batch Prep   │                │  - gettxoutproof│
└─────────────────┘                └─────────────────┘
         │
         ▼
┌─────────────────┐
│   ZisK zkVM     │
│                 │
│  - main.rs      │
│  - validation   │
│  - STARK proof  │
└─────────────────┘
```

## 🔧 **1. Host-Side Setup**

### **Dependencies**

Add to your `Cargo.toml`:

```toml
[dependencies]
zisk-zcash-validator = { path = "../zisk_integration", features = ["host"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
serde_json = "1.0"
anyhow = "1.0"
```

### **Configuration**

```rust
use zisk_zcash_validator::bridge::node_bridge::{NodeBridge, NodeConfig};

// Production configuration
let config = NodeConfig {
    rpc_url: "https://zcash-node.example.com:8232".to_string(),
    rpc_user: "production_user".to_string(),
    rpc_password: "secure_password".to_string(),
    cookie_file: None, // Use user/password auth
    timeout: 60,
    max_retries: 5,
    verify_tls: true,
    proxy_url: None,
    pool_size: 20,
};

// Or use cookie authentication
let config = NodeConfig {
    rpc_url: "https://zcash-node.example.com:8232".to_string(),
    rpc_user: "".to_string(), // Empty for cookie auth
    rpc_password: "".to_string(),
    cookie_file: Some("~/.zcash/.cookie".to_string()),
    timeout: 60,
    max_retries: 5,
    verify_tls: true,
    proxy_url: None,
    pool_size: 20,
};
```

### **Host Application**

```rust
use zisk_zcash_validator::bridge::{NodeBridge, ZkvmInputSerializer};
use zisk_zcash_validator::core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize node bridge
    let mut bridge = NodeBridge::new(config);
    bridge.initialize_client().await?;
    
    // Create input serializer
    let mut serializer = ZkvmInputSerializer::new(bridge);
    
    // Prepare transaction batch
    let transactions = load_transactions_from_source().await?;
    let prior_state_root = get_current_state_root().await?;
    
    // Serialize for zkVM
    let batch_input = serializer.serialize_batch_input(&transactions, prior_state_root).await?;
    let binary_input = serializer.to_binary(&batch_input)?;
    
    // Run zkVM validation
    let proof = run_zkvm_validation(binary_input).await?;
    
    // Verify proof
    verify_stark_proof(&proof)?;
    
    // Update state
    update_state_root(batch_input.new_state_root).await?;
    
    Ok(())
}
```

## 🔧 **2. Zcash Node Setup**

### **Mainnet Configuration**

```bash
# zcash.conf
rpcuser=your_username
rpcpassword=your_secure_password
rpcbind=0.0.0.0
rpcallowip=0.0.0.0/0
rpcport=8232
server=1
daemon=1
```

### **Testnet Configuration**

```bash
# zcash.conf
testnet=1
rpcuser=testnet_user
rpcpassword=testnet_password
rpcbind=0.0.0.0
rpcallowip=0.0.0.0/0
rpcport=18232
server=1
daemon=1
```

### **Regtest Configuration (for testing)**

```bash
# zcash.conf
regtest=1
rpcuser=user
rpcpassword=pass
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
rpcport=8232
server=1
daemon=1
```

## 🔧 **3. Security Considerations**

### **Authentication**

1. **Use strong passwords** for RPC authentication
2. **Enable TLS** in production (`verify_tls: true`)
3. **Use cookie authentication** when possible (more secure)
4. **Restrict RPC access** to trusted IPs only

### **Network Security**

```rust
// Production config with security
let config = NodeConfig {
    rpc_url: "https://zcash-node.example.com:8232".to_string(),
    rpc_user: "".to_string(),
    rpc_password: "".to_string(),
    cookie_file: Some("/secure/path/.zcash/.cookie".to_string()),
    timeout: 30,
    max_retries: 3,
    verify_tls: true, // CRITICAL for production
    proxy_url: None,
    pool_size: 10,
};
```

### **Error Handling**

```rust
use zisk_zcash_validator::error::*;

match bridge.fetch_utxo(txid, vout).await {
    Ok(utxo) => {
        // Process UTXO
    },
    Err(ZcashValidationError::SystemError(SystemError::RpcError(msg))) => {
        // Handle RPC errors
        eprintln!("RPC error: {}", msg);
    },
    Err(ZcashValidationError::SystemError(SystemError::NetworkError(msg))) => {
        // Handle network errors
        eprintln!("Network error: {}", msg);
    },
    Err(e) => {
        // Handle other errors
        eprintln!("Unexpected error: {}", e);
    }
}
```

## 🔧 **4. Performance Optimization**

### **Batch Processing**

```rust
// Fetch multiple UTXOs in parallel
use futures::stream::FuturesUnordered;

async fn fetch_utxos_parallel(
    bridge: &NodeBridge,
    utxo_refs: &[([u8; 32], u32)]
) -> Result<Vec<NodeUtxo>, ZcashValidationError> {
    let mut futures = FuturesUnordered::new();
    
    for (txid, vout) in utxo_refs {
        futures.push(bridge.fetch_utxo(*txid, *vout));
    }
    
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        results.push(result?);
    }
    
    Ok(results)
}
```

### **Caching**

```rust
// Implement UTXO caching
struct CachedNodeBridge {
    bridge: NodeBridge,
    cache: HashMap<([u8; 32], u32), (NodeUtxo, Instant)>,
    cache_ttl: Duration,
}

impl CachedNodeBridge {
    async fn fetch_utxo_cached(&mut self, txid: [u8; 32], vout: u32) -> ZcashResult<NodeUtxo> {
        let key = (txid, vout);
        
        // Check cache first
        if let Some((utxo, timestamp)) = self.cache.get(&key) {
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(utxo.clone());
            }
        }
        
        // Fetch from node
        let utxo = self.bridge.fetch_utxo(txid, vout).await?;
        
        // Cache the result
        self.cache.insert(key, (utxo.clone(), Instant::now()));
        
        Ok(utxo)
    }
}
```

## 🔧 **5. Monitoring & Logging**

### **Structured Logging**

```rust
use tracing::{info, warn, error, instrument};

#[instrument]
async fn process_transaction_batch(
    bridge: &mut NodeBridge,
    transactions: &[ZcashTransaction]
) -> Result<ValidationResult, ZcashValidationError> {
    info!("Processing batch of {} transactions", transactions.len());
    
    let start = Instant::now();
    let result = bridge.fetch_utxos_batch(&utxo_refs).await?;
    let duration = start.elapsed();
    
    info!("Fetched {} UTXOs in {:?}", result.len(), duration);
    
    // Process transactions...
    
    Ok(validation_result)
}
```

### **Metrics Collection**

```rust
use prometheus::{Counter, Histogram, Registry};

struct Metrics {
    rpc_calls_total: Counter,
    rpc_duration: Histogram,
    utxos_fetched_total: Counter,
    validation_errors_total: Counter,
}

impl Metrics {
    fn new(registry: &Registry) -> Self {
        Self {
            rpc_calls_total: Counter::new("rpc_calls_total", "Total RPC calls").unwrap(),
            rpc_duration: Histogram::new("rpc_duration_seconds", "RPC call duration").unwrap(),
            utxos_fetched_total: Counter::new("utxos_fetched_total", "Total UTXOs fetched").unwrap(),
            validation_errors_total: Counter::new("validation_errors_total", "Total validation errors").unwrap(),
        }
    }
}
```

## 🔧 **6. Testing**

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    
    #[tokio::test]
    async fn test_rpc_call_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "result": {
                        "value": "1.0",
                        "scriptPubKey": {
                            "hex": "76a914000000000000000000000000000000000000000088ac"
                        },
                        "height": 100,
                        "spendable": true
                    }
                })))
            .mount(&mock_server)
            .await;
        
        let config = NodeConfig {
            rpc_url: mock_server.uri(),
            // ... other config
        };
        
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().await.unwrap();
        
        let result = bridge.fetch_utxo([0u8; 32], 0).await;
        assert!(result.is_ok());
    }
}
```

### **Integration Tests**

```bash
# Start regtest node
zcashd -regtest -daemon -rpcuser=user -rpcpassword=pass

# Mine blocks
zcash-cli -regtest -rpcuser=user -rpcpassword=pass generate 100

# Create test transactions
zcash-cli -regtest -rpcuser=user -rpcpassword=pass sendtoaddress "t1..." 1.0

# Run integration tests
cargo test --features host
```

## 🔧 **7. Deployment Checklist**

### **Pre-deployment**

- [ ] **Node Configuration**: Zcash node properly configured and running
- [ ] **Authentication**: RPC credentials or cookie file set up
- [ ] **TLS**: HTTPS enabled for production
- [ ] **Firewall**: RPC port accessible from host application
- [ ] **Monitoring**: Logging and metrics configured
- [ ] **Testing**: All tests passing with real node

### **Deployment**

- [ ] **Build**: Compile with `--release` and `--features host`
- [ ] **Configuration**: Production config files in place
- [ ] **Secrets**: Secure credential management
- [ ] **Health Checks**: Endpoint monitoring configured
- [ ] **Rollback Plan**: Backup and rollback procedures ready

### **Post-deployment**

- [ ] **Monitoring**: Verify metrics and logs
- [ ] **Performance**: Check RPC call latency
- [ ] **Error Rates**: Monitor validation success rates
- [ ] **State Sync**: Verify state root consistency
- [ ] **Proof Generation**: Test STARK proof generation

## 🚀 **Production Ready!**

Your ZisK-Zcash validator is now production-ready with:

- ✅ **Real RPC Integration**: Connects to actual Zcash nodes
- ✅ **Robust Error Handling**: Handles network failures gracefully
- ✅ **Security**: TLS, authentication, and input validation
- ✅ **Performance**: Caching, batching, and parallel processing
- ✅ **Monitoring**: Comprehensive logging and metrics
- ✅ **Testing**: Unit and integration test coverage

**Next Steps**: Deploy to your production environment and start validating real Zcash transactions! 🎯
