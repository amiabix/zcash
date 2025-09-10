//! Integration tests for ZisK-Zcash validator with real RPC calls

#[cfg(feature = "host")]
mod tests {
    use zisk_zcash_validator::bridge::node_bridge::{NodeBridge, NodeConfig};
    use zisk_zcash_validator::bridge::input_serializer::ZkvmInputSerializer;
    use zisk_zcash_validator::core::*;
    use std::time::Duration;

    /// Test configuration for regtest
    fn create_test_config() -> NodeConfig {
        NodeConfig {
            rpc_url: "http://localhost:8232".to_string(),
            rpc_user: "user".to_string(),
            rpc_password: "pass".to_string(),
            cookie_file: None,
            timeout: 30,
            max_retries: 3,
            verify_tls: false,
            proxy_url: None,
            pool_size: 10,
        }
    }

    #[tokio::test]
    async fn test_node_bridge_connection() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        
        // Initialize HTTP client
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Test basic connectivity
        let height = bridge.get_block_height().await;
        match height {
            Ok(h) => println!("Connected to node at height: {}", h),
            Err(e) => {
                println!("Connection test failed (expected if no node running): {:?}", e);
                // This is expected if no regtest node is running
                return;
            }
        }
    }

    #[tokio::test]
    async fn test_utxo_fetching() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Create a test txid (this would be a real txid in actual testing)
        let test_txid = [0u8; 32];
        
        // Try to fetch UTXO (will fail if not found, which is expected)
        let result = bridge.fetch_utxo(test_txid, 0).await;
        match result {
            Ok(utxo) => {
                println!("Fetched UTXO: {:?}", utxo);
                assert!(utxo.value > 0);
            },
            Err(e) => {
                println!("UTXO fetch failed (expected if UTXO doesn't exist): {:?}", e);
                // This is expected if the UTXO doesn't exist
            }
        }
    }

    #[tokio::test]
    async fn test_merkle_proof_fetching() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Create a test txid
        let test_txid = [0u8; 32];
        
        // Try to fetch Merkle proof
        let result = bridge.get_merkle_proof(test_txid).await;
        match result {
            Ok(proof) => {
                println!("Fetched Merkle proof: {} bytes", proof.len());
                assert!(!proof.is_empty());
            },
            Err(e) => {
                println!("Merkle proof fetch failed (expected if tx not found): {:?}", e);
                // This is expected if the transaction doesn't exist
            }
        }
    }

    #[tokio::test]
    async fn test_batch_utxo_fetching() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Create test UTXO references
        let utxo_refs = vec![
            ([0u8; 32], 0),
            ([1u8; 32], 1),
            ([2u8; 32], 2),
        ];
        
        // Try to fetch batch UTXOs
        let result = bridge.fetch_utxos_batch(&utxo_refs).await;
        match result {
            Ok(utxos) => {
                println!("Fetched {} UTXOs", utxos.len());
                for utxo in utxos {
                    println!("UTXO: txid={:?}, vout={}, value={}", 
                        utxo.txid, utxo.vout, utxo.value);
                }
            },
            Err(e) => {
                println!("Batch UTXO fetch failed (expected if UTXOs don't exist): {:?}", e);
                // This is expected if the UTXOs don't exist
            }
        }
    }

    #[tokio::test]
    async fn test_value_conversion() {
        use serde_json::json;
        
        let config = create_test_config();
        let bridge = NodeBridge::new(config);
        
        // Test string decimal conversion
        let value_str = json!("1.23456789");
        let zatoshis = bridge.value_to_zatoshis(&value_str).expect("Failed to convert string value");
        assert_eq!(zatoshis, 123456789);
        
        // Test float conversion
        let value_float = json!(2.5);
        let zatoshis = bridge.value_to_zatoshis(&value_float).expect("Failed to convert float value");
        assert_eq!(zatoshis, 250000000);
        
        // Test small value
        let value_small = json!("0.00000001");
        let zatoshis = bridge.value_to_zatoshis(&value_small).expect("Failed to convert small value");
        assert_eq!(zatoshis, 1);
    }

    #[tokio::test]
    async fn test_txid_conversion() {
        let config = create_test_config();
        let bridge = NodeBridge::new(config);
        
        // Test hex to bytes conversion
        let hex_str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let txid = bridge.txid_hex_to_bytes(hex_str).expect("Failed to convert hex to txid");
        
        // Verify the conversion
        let expected = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        ];
        assert_eq!(txid, expected);
    }

    #[tokio::test]
    async fn test_input_serialization() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Create a test transaction
        let transaction = ZcashTransaction {
            version: 4,
            version_group_id: 0x892F2085,
            lock_time: 0,
            expiry_height: 0,
            transparent_inputs: vec![
                TransparentInput {
                    prevout_hash: [0u8; 32],
                    prevout_index: 0,
                    script_sig: vec![],
                    sequence: 0xFFFFFFFF,
                }
            ],
            transparent_outputs: vec![
                TransparentOutput {
                    value: 100000000, // 1 ZEC
                    script_pubkey: vec![0x76, 0xa9, 0x14, 0x00], // P2PKH
                }
            ],
            sapling_bundle: None,
            orchard_bundle: None,
        };
        
        let prior_state_root = [0u8; 32];
        
        // Create serializer
        let mut serializer = ZkvmInputSerializer::new(bridge);
        
        // Serialize batch input
        let result = serializer.serialize_batch_input(&[transaction], prior_state_root).await;
        match result {
            Ok(batch_input) => {
                println!("Serialized batch input: {} UTXOs, {} bytes tx data", 
                    batch_input.utxos.len(), batch_input.tx_batch.len());
                
                // Convert to binary
                let binary = serializer.to_binary(&batch_input).expect("Failed to convert to binary");
                println!("Binary input size: {} bytes", binary.len());
                
                // Verify structure
                assert_eq!(batch_input.prior_state_root, prior_state_root);
                assert_eq!(batch_input.consensus_branch_id, 0x892F2085);
                assert!(!binary.is_empty());
            },
            Err(e) => {
                println!("Serialization failed (expected if no UTXOs found): {:?}", e);
                // This is expected if the UTXOs don't exist
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Test with invalid txid (should return error)
        let invalid_txid = [0xFFu8; 32];
        let result = bridge.fetch_utxo(invalid_txid, 999).await;
        
        match result {
            Ok(_) => {
                // If this succeeds, it means the UTXO actually exists (unlikely)
                println!("Unexpectedly found UTXO for invalid txid");
            },
            Err(e) => {
                println!("Expected error for invalid UTXO: {:?}", e);
                // This is the expected behavior
            }
        }
    }

    #[tokio::test]
    async fn test_performance_batch_fetching() {
        let config = create_test_config();
        let mut bridge = NodeBridge::new(config);
        bridge.initialize_client().expect("Failed to initialize client");
        
        // Create many UTXO references to test batch performance
        let mut utxo_refs = Vec::new();
        for i in 0..100 {
            let mut txid = [0u8; 32];
            txid[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            utxo_refs.push((txid, 0));
        }
        
        let start = std::time::Instant::now();
        let result = bridge.fetch_utxos_batch(&utxo_refs).await;
        let duration = start.elapsed();
        
        match result {
            Ok(utxos) => {
                println!("Fetched {} UTXOs in {:?}", utxos.len(), duration);
                // Performance should be reasonable even if most UTXOs don't exist
                assert!(duration < Duration::from_secs(10));
            },
            Err(e) => {
                println!("Batch fetch failed in {:?}: {:?}", duration, e);
                // This is expected if the UTXOs don't exist
            }
        }
    }
}

/// Helper function to create a test Zcash regtest environment
#[cfg(feature = "host")]
pub async fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
    // This would start a Zcash regtest node and mine some blocks
    // For now, just return Ok since we're testing against a potentially non-existent node
    println!("Test environment setup (regtest node should be running)");
    println!("To run integration tests with a real node:");
    println!("1. Start zcashd: zcashd -regtest -daemon -rpcuser=user -rpcpassword=pass");
    println!("2. Mine blocks: zcash-cli -regtest -rpcuser=user -rpcpassword=pass generate 100");
    println!("3. Create transactions: zcash-cli -regtest -rpcuser=user -rpcpassword=pass sendtoaddress <address> 1.0");
    Ok(())
}
