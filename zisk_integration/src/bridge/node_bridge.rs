//! Zcash Node Bridge for UTXO Integration
//! 
//! This module provides real-time integration with Zcash nodes to fetch
//! actual UTXO data and maintain state consistency.

use crate::core::*;
use crate::error::*;
use alloc::vec::Vec;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use serde_json::json;
use alloc::collections::BTreeMap;
use sha2::{Sha256, Digest};
use hex;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Zcash node connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node RPC endpoint
    pub rpc_url: String,
    /// RPC credentials (leave empty to use cookie auth)
    pub rpc_user: String,
    pub rpc_password: String,
    /// Cookie file path (alternative to user/password)
    pub cookie_file: Option<String>,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Enable TLS verification
    pub verify_tls: bool,
    /// Enable proxy support
    pub proxy_url: Option<String>,
    /// Connection pool size
    pub pool_size: usize,
}

/// UTXO data fetched from Zcash node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUtxo {
    /// Transaction ID
    pub txid: [u8; 32],
    /// Output index
    pub vout: u32,
    /// Value in zatoshis
    pub value: u64,
    /// Script public key
    pub script_pubkey: Vec<u8>,
    /// Block height when UTXO was created
    pub height: u32,
    /// Whether UTXO is spendable
    pub spendable: bool,
}

/// Sapling note data from node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSaplingNote {
    /// Note commitment
    pub cmu: [u8; 32],
    /// Value commitment
    pub cv: [u8; 32],
    /// Ephemeral public key
    pub ephemeral_key: [u8; 32],
    /// Encrypted ciphertext
    pub enc_ciphertext: Vec<u8>,
    /// Out ciphertext
    pub out_ciphertext: Vec<u8>,
    /// Block height
    pub height: u32,
}

/// Orchard note data from node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOrchardNote {
    /// Note commitment
    pub cmu: [u8; 32],
    /// Value commitment
    pub cv: [u8; 32],
    /// Ephemeral public key
    pub ephemeral_key: [u8; 32],
    /// Encrypted ciphertext
    pub enc_ciphertext: Vec<u8>,
    /// Out ciphertext
    pub out_ciphertext: Vec<u8>,
    /// Block height
    pub height: u32,
}

/// Zcash node bridge for real-time data fetching
pub struct NodeBridge {
    config: NodeConfig,
    /// Current block height
    current_height: u32,
    /// UTXO cache for performance
    utxo_cache: BTreeMap<([u8; 32], u32), NodeUtxo>,
    /// Sapling note cache
    sapling_cache: BTreeMap<[u8; 32], NodeSaplingNote>,
    /// Orchard note cache
    orchard_cache: BTreeMap<[u8; 32], NodeOrchardNote>,
    /// HTTP client for RPC calls
    client: Option<reqwest::Client>,
}

impl NodeBridge {
    /// Create a new node bridge
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            current_height: 0,
            utxo_cache: BTreeMap::new(),
            sapling_cache: BTreeMap::new(),
            orchard_cache: BTreeMap::new(),
            client: None,
        }
    }

    /// Initialize the HTTP client (call this before making RPC calls)
    pub fn initialize_client(&mut self) -> ZcashResult<()> {
        // Build reqwest client with TLS, timeout, etc.
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout.max(10)));

        if !self.config.verify_tls {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Add proxy if configured
        if let Some(ref proxy_url) = self.config.proxy_url {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }

        // Add connection pool configuration
        builder = builder.pool_max_idle_per_host(self.config.pool_size);

        let client = builder.build()
            .map_err(|e| ZcashValidationError::SystemError(SystemError::NetworkError(format!("Failed to create HTTP client: {:?}", e))))?;
        
        self.client = Some(client);
        Ok(())
    }

    /// Make an RPC call to the Zcash node
    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> ZcashResult<serde_json::Value> {
        let client = self.client.as_ref()
            .ok_or_else(|| ZcashValidationError::SystemError(SystemError::ConfigError("HTTP client not initialized".into())))?;

        // JSON RPC body
        let body = json!({
            "jsonrpc": "1.0",
            "id": "zisk",
            "method": method,
            "params": params,
        });

        // Auth: prefer cookie if rpc_user empty
        let mut req = client.post(&self.config.rpc_url).json(&body);

        if !self.config.rpc_user.is_empty() {
            req = req.basic_auth(&self.config.rpc_user, Some(&self.config.rpc_password));
        } else if let Some(ref cookie_file) = self.config.cookie_file {
            // Read cookie file and set Authorization header
            let cookie_auth = self.read_cookie_file(cookie_file)?;
            req = req.header("Authorization", format!("Basic {}", cookie_auth));
        }

        // Retry loop with exponential backoff
        let mut backoff = 100u64;
        for attempt in 0..=self.config.max_retries {
            let resp = req.try_clone().unwrap().send().await;

            match resp {
                Ok(mut r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    
                    if !status.is_success() {
                        // Try to parse error JSON
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(err) = v.get("error") {
                                if !err.is_null() {
                                    return Err(ZcashValidationError::SystemError(SystemError::RpcError(format!("{:?}", err))));
                                }
                            }
                            return Ok(v["result"].clone());
                        } else {
                            return Err(ZcashValidationError::SystemError(SystemError::NetworkError(format!("RPC HTTP error {}: {}", status, text))));
                        }
                    } else {
                        let v: serde_json::Value = serde_json::from_str(&text)
                            .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Failed to parse JSON response: {:?}", e))))?;
                        
                        if let Some(err) = v.get("error") {
                            if !err.is_null() {
                                return Err(ZcashValidationError::SystemError(SystemError::RpcError(format!("RPC error: {:?}", err))));
                            }
                        }
                        return Ok(v["result"].clone());
                    }
                }
                Err(e) => {
                    if attempt == self.config.max_retries {
                        return Err(ZcashValidationError::SystemError(SystemError::NetworkError(format!("RPC failed after {} retries: {:?}", self.config.max_retries, e))));
                    }
                    // Exponential backoff and retry
                    tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff = (backoff * 2).min(5000);
                    continue;
                }
            }
        }

        Err(ZcashValidationError::SystemError(SystemError::NetworkError("Exhausted retries".into())))
    }

    /// Read cookie file for authentication
    fn read_cookie_file(&self, cookie_file: &str) -> ZcashResult<String> {
        use std::fs;
        use base64::encode;
        
        let cookie_content = fs::read_to_string(cookie_file)
            .map_err(|e| ZcashValidationError::SystemError(SystemError::FileNotFound(format!("Failed to read cookie file {}: {:?}", cookie_file, e))))?;
        
        // Cookie format is usually "user:password" or just "password"
        let auth_string = if cookie_content.contains(':') {
            cookie_content.trim().to_string()
        } else {
            format!(":{}", cookie_content.trim())
        };
        
        Ok(encode(auth_string))
    }

    /// Fetch UTXO data from Zcash node
    pub async fn fetch_utxo(&mut self, txid: [u8; 32], vout: u32) -> ZcashResult<NodeUtxo> {
        // Check cache first
        if let Some(utxo) = self.utxo_cache.get(&(txid, vout)) {
            return Ok(utxo.clone());
        }

        // Fetch from node via RPC
        let utxo = self.rpc_get_utxo(txid, vout).await?;
        
        // Cache the result
        self.utxo_cache.insert((txid, vout), utxo.clone());
        
        Ok(utxo)
    }

    /// Fetch multiple UTXOs in batch
    pub async fn fetch_utxos_batch(&mut self, utxo_refs: &[([u8; 32], u32)]) -> ZcashResult<Vec<NodeUtxo>> {
        let mut utxos = Vec::new();
        let mut uncached_refs = Vec::new();

        // Check cache for existing UTXOs
        for (txid, vout) in utxo_refs {
            if let Some(utxo) = self.utxo_cache.get(&(*txid, *vout)) {
                utxos.push(utxo.clone());
            } else {
                uncached_refs.push((*txid, *vout));
            }
        }

        // Fetch uncached UTXOs from node
        if !uncached_refs.is_empty() {
            let fetched_utxos = self.rpc_get_utxos_batch(&uncached_refs).await?;
            
            // Cache and add to results
            for utxo in fetched_utxos {
                self.utxo_cache.insert((utxo.txid, utxo.vout), utxo.clone());
                utxos.push(utxo);
            }
        }

        Ok(utxos)
    }

    /// Get current block height
    pub async fn get_block_height(&mut self) -> ZcashResult<u32> {
        let height = self.rpc_get_block_count().await?;
        self.current_height = height;
        Ok(height)
    }

    /// Get current state root (UTXO set root)
    pub async fn get_state_root(&mut self) -> ZcashResult<[u8; 32]> {
        // This would call a custom RPC method to get the UTXO set root
        // For now, we'll compute it from our cache
        self.compute_state_root_from_cache()
    }

    /// Update state after processing transactions
    pub async fn update_state(&mut self, spent_utxos: &[([u8; 32], u32)], new_utxos: &[NodeUtxo]) -> ZcashResult<()> {
        // Remove spent UTXOs from cache
        for (txid, vout) in spent_utxos {
            self.utxo_cache.remove(&(*txid, *vout));
        }

        // Add new UTXOs to cache
        for utxo in new_utxos {
            self.utxo_cache.insert((utxo.txid, utxo.vout), utxo.clone());
        }

        Ok(())
    }

    /// RPC call to get single UTXO
    async fn rpc_get_utxo(&self, txid: [u8; 32], vout: u32) -> ZcashResult<NodeUtxo> {
        // Convert txid to hex string (big-endian)
        let txid_hex = hex::encode(txid);
        
        // Make RPC call to gettxout
        let response = self.rpc_call("gettxout", json!([txid_hex, vout])).await?;
        
        if response.is_null() {
            return Err(ZcashValidationError::SystemError(
                SystemError::FileNotFound(format!("UTXO not found: {}:{}", txid_hex, vout))
            ));
        }
        
        // Parse response with proper value conversion
        let value = self.value_to_zatoshis(&response["value"])?;
        let script_pubkey = hex::decode(response["scriptPubKey"]["hex"].as_str().unwrap_or(""))
            .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid script hex: {:?}", e))))?;
        let height = response["height"].as_u64().unwrap_or(0) as u32;
        let spendable = response["spendable"].as_bool().unwrap_or(false);
        
        Ok(NodeUtxo {
            txid,
            vout,
            value,
            script_pubkey,
            height,
            spendable,
        })
    }

    /// RPC call to get multiple UTXOs
    async fn rpc_get_utxos_batch(&self, utxo_refs: &[([u8; 32], u32)]) -> ZcashResult<Vec<NodeUtxo>> {
        let mut utxos = Vec::new();
        for (txid, vout) in utxo_refs {
            utxos.push(self.rpc_get_utxo(*txid, *vout).await?);
        }
        Ok(utxos)
    }

    /// RPC call to get block count
    async fn rpc_get_block_count(&self) -> ZcashResult<u32> {
        let response = self.rpc_call("getblockcount", json!([])).await?;
        Ok(response.as_u64().unwrap_or(0) as u32)
    }

    /// Compute state root from current cache
    fn compute_state_root_from_cache(&self) -> ZcashResult<[u8; 32]> {
        // This would compute the actual UTXO set root
        // For now, return a hash of all UTXOs
        let mut combined = Vec::new();
        for ((txid, vout), utxo) in &self.utxo_cache {
            combined.extend_from_slice(txid);
            combined.extend_from_slice(&vout.to_le_bytes());
            combined.extend_from_slice(&utxo.value.to_le_bytes());
        }
        
        Ok(sha2::Sha256::digest(&combined).into())
    }

    /// Convert RPC value to zatoshis (avoiding float rounding)
    fn value_to_zatoshis(&self, value: &serde_json::Value) -> ZcashResult<u64> {
        if value.is_string() {
            let s = value.as_str().unwrap();
            let dec = Decimal::from_str_exact(s)
                .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid decimal value: {:?}", e))))?;
            // 1 ZEC = 100_000_000 zatoshis
            let sat = (dec * Decimal::new(100_000_000, 0)).round();
            sat.to_u64()
                .ok_or_else(|| ZcashValidationError::SystemError(SystemError::ParseError("Value too large for u64".into())))
        } else if value.is_f64() {
            // Fallback if node returns float (less safe)
            let f = value.as_f64().unwrap();
            Ok((f * 100_000_000.0).round() as u64)
        } else {
            Err(ZcashValidationError::SystemError(SystemError::ParseError("Invalid value format".into())))
        }
    }

    /// Convert hex string to txid bytes (handles big-endian hex)
    fn txid_hex_to_bytes(&self, txid_hex: &str) -> ZcashResult<[u8; 32]> {
        let cleaned_hex = txid_hex.trim_start_matches("0x");
        let bytes = hex::decode(cleaned_hex)
            .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid hex: {:?}", e))))?;
        
        if bytes.len() != 32 {
            return Err(ZcashValidationError::SystemError(SystemError::ParseError("Invalid txid length".into())));
        }
        
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }

    /// Get Merkle proof for UTXO
    pub async fn get_merkle_proof(&self, txid: [u8; 32]) -> ZcashResult<Vec<u8>> {
        let txid_hex = hex::encode(txid);
        let response = self.rpc_call("gettxoutproof", json!([[txid_hex]])).await?;
        
        let proof_hex = response.as_str()
            .ok_or_else(|| ZcashValidationError::SystemError(SystemError::ParseError("Invalid proof format".into())))?;
        
        hex::decode(proof_hex)
            .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid proof hex: {:?}", e))))
    }

    /// Get raw transaction data
    pub async fn get_raw_transaction(&self, txid: [u8; 32]) -> ZcashResult<Vec<u8>> {
        let txid_hex = hex::encode(txid);
        let response = self.rpc_call("getrawtransaction", json!([txid_hex, 1])).await?;
        
        let hex_data = response["hex"].as_str()
            .ok_or_else(|| ZcashValidationError::SystemError(SystemError::ParseError("No hex data in response".into())))?;
        
        hex::decode(hex_data)
            .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid transaction hex: {:?}", e))))
    }

    /// Get block data
    pub async fn get_block(&self, block_hash: [u8; 32]) -> ZcashResult<serde_json::Value> {
        let hash_hex = hex::encode(block_hash);
        self.rpc_call("getblock", json!([hash_hex, 1])).await
    }

    /// Get block hash by height
    pub async fn get_block_hash(&self, height: u32) -> ZcashResult<[u8; 32]> {
        let response = self.rpc_call("getblockhash", json!([height])).await?;
        let hash_hex = response.as_str()
            .ok_or_else(|| ZcashValidationError::SystemError(SystemError::ParseError("Invalid block hash format".into())))?;
        
        self.txid_hex_to_bytes(hash_hex)
    }
}

/// UTXO provider trait for dependency injection
pub trait UtxoProvider {
    async fn get_utxo(&mut self, txid: [u8; 32], vout: u32) -> ZcashResult<NodeUtxo>;
    async fn get_utxos_batch(&mut self, utxo_refs: &[([u8; 32], u32)]) -> ZcashResult<Vec<NodeUtxo>>;
    async fn get_state_root(&mut self) -> ZcashResult<[u8; 32]>;
}

impl UtxoProvider for NodeBridge {
    async fn get_utxo(&mut self, txid: [u8; 32], vout: u32) -> ZcashResult<NodeUtxo> {
        self.fetch_utxo(txid, vout).await
    }

    async fn get_utxos_batch(&mut self, utxo_refs: &[([u8; 32], u32)]) -> ZcashResult<Vec<NodeUtxo>> {
        self.fetch_utxos_batch(utxo_refs).await
    }

    async fn get_state_root(&mut self) -> ZcashResult<[u8; 32]> {
        self.get_state_root().await
    }
}
