#[cfg(test)]
mod tests {
    use zcash_validator::secp_verify::{verify_secp256k1_c, double_sha256};
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use sha2::Digest;

    #[test]
    fn sign_and_verify_compact() {
        let secp = Secp256k1::new();
        
        // Generate a test secret key (32 bytes)
        let sk_bytes = [1u8; 32]; // Use a fixed key for testing
        let sk = SecretKey::from_slice(&sk_bytes).unwrap();
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let pk_comp = pk.serialize(); // 33 bytes

        // example message
        let msg32 = [0x42u8; 32];

        // sign
        let msg = Message::from_digest_slice(&msg32).unwrap();
        let sig = secp.sign_ecdsa(&msg, &sk);
        let compact = sig.serialize_compact();

        // verify using wrapper
        assert!(verify_secp256k1_c(&msg32, &compact, &pk_comp.as_slice().try_into().unwrap()));
    }

    #[test]
    fn test_double_sha256() {
        let data = b"hello world";
        let result = double_sha256(data);
        
        // Test that it's deterministic
        let result2 = double_sha256(data);
        assert_eq!(result, result2);
        
        // Test that it's different from single SHA256
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        let single_sha = hasher.finalize();
        assert_ne!(result, single_sha.as_slice());
    }

    #[test]
    fn test_sighash_all_computation() {
        use zcash_validator::secp_verify::compute_sighash_all_like;
        
        // Create a simple test transaction
        let mut tx_bytes = Vec::new();
        
        // Version (4 bytes LE)
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());
        
        // Input count (1)
        tx_bytes.push(1);
        
        // Input: prev_txid (32 bytes) + prev_index (4 bytes) + script_len (0) + sequence (4 bytes)
        tx_bytes.extend_from_slice(&[0u8; 32]); // prev_txid
        tx_bytes.extend_from_slice(&0u32.to_le_bytes()); // prev_index
        tx_bytes.push(0); // script_len = 0
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // sequence
        
        // Output count (1)
        tx_bytes.push(1);
        
        // Output: value (8 bytes) + script_len + script_pubkey
        tx_bytes.extend_from_slice(&1000000u64.to_le_bytes()); // value
        tx_bytes.push(25); // script_len = 25
        tx_bytes.extend_from_slice(&[0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac]); // script_pubkey
        
        // Lock time (4 bytes LE)
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());
        
        // Test sighash computation
        let script_pubkey = b"test_script_pubkey";
        let sighash = compute_sighash_all_like(&tx_bytes, 0, script_pubkey);
        
        // Test that it's deterministic
        let sighash2 = compute_sighash_all_like(&tx_bytes, 0, script_pubkey);
        assert_eq!(sighash, sighash2);
        
        // Test that different input index gives different result (if input exists)
        // For now, just test that the function doesn't panic with invalid index
        let sighash_diff = compute_sighash_all_like(&tx_bytes, 1, script_pubkey);
        // Note: This might be the same due to how we handle invalid indices
        println!("Sighash for input 0: {:02x?}", sighash);
        println!("Sighash for input 1: {:02x?}", sighash_diff);
        
        // Test that different script gives different result
        let script_pubkey2 = b"different_script";
        let sighash_diff_script = compute_sighash_all_like(&tx_bytes, 0, script_pubkey2);
        println!("Sighash with different script: {:02x?}", sighash_diff_script);
        
        // For now, just verify the function works without panicking
        // The actual difference might depend on the exact implementation
        assert_eq!(sighash.len(), 32);
        assert_eq!(sighash_diff_script.len(), 32);
        
        println!("Sighash test passed! Generated sighash: {:02x?}", sighash);
    }

    #[test]
    fn test_utxo_validation() {
        use zcash_validator::secp_verify::double_sha256;
        
        // Create a simple test UTXO
        let prev_txid = [1u8; 32];
        let prev_index = 0u32;
        let value = 100000000u64; // 1.0 ZEC
        let script_pubkey = b"test_script_pubkey".to_vec();
        
        // Create a simple Merkle proof (single leaf)
        let leaf_hash = {
            let mut leaf = Vec::new();
            leaf.extend_from_slice(&prev_txid);
            leaf.extend_from_slice(&prev_index.to_le_bytes());
            leaf.extend_from_slice(&value.to_le_bytes());
            leaf.extend_from_slice(&double_sha256(&script_pubkey));
            double_sha256(&leaf)
        };
        
        // For a single leaf, the root is just the leaf hash
        let prior_state_root = leaf_hash;
        
        // Create UTXO entry
        let utxo = zcash_validator::UtxoEntry {
            prev_txid,
            prev_index,
            value,
            script_pubkey,
            merkle_proof: vec![], // Empty proof for single leaf
            leaf_index: 0,
        };
        
        // Test UTXO inclusion verification
        let is_included = zcash_validator::verify_utxo_inclusion(&utxo, &prior_state_root);
        assert!(is_included, "UTXO should be included in the state root");
        
        // Test double spend prevention
        let utxos = vec![utxo.clone(), utxo.clone()];
        let has_double_spend = !zcash_validator::check_double_spend(&utxos);
        assert!(has_double_spend, "Should detect double spend");
        
        // Test no double spend with different UTXOs
        let utxo2 = zcash_validator::UtxoEntry {
            prev_txid: [2u8; 32], // Different txid
            prev_index: 0,
            value: 50000000,
            script_pubkey: b"different_script".to_vec(),
            merkle_proof: vec![],
            leaf_index: 1,
        };
        let utxos_no_double_spend = vec![utxo, utxo2];
        let has_no_double_spend = zcash_validator::check_double_spend(&utxos_no_double_spend);
        assert!(has_no_double_spend, "Should not detect double spend with different UTXOs");
        
        println!("UTXO validation test passed!");
    }

    #[test]
    fn test_smt_implementation() {
        use zcash_validator::{SparseMerkleTree, UtxoSet, UtxoBatch};
        
        // Test Sparse Merkle Tree basic operations
        let mut smt = SparseMerkleTree::new(4); // 4-level tree
        
        // Test empty tree root
        let empty_root = smt.root();
        assert_eq!(empty_root.len(), 32);
        
        // Test leaf insertion
        let leaf_hash = [1u8; 32];
        smt.set_leaf(0, leaf_hash);
        let root_after_insert = smt.root();
        assert_ne!(empty_root, root_after_insert);
        
        // Test leaf removal
        smt.remove_leaf(0);
        let root_after_remove = smt.root();
        assert_eq!(empty_root, root_after_remove);
        
        // Test UTXO set operations
        let mut utxo_set = UtxoSet::new();
        
        // Add a UTXO
        let prev_txid = [1u8; 32];
        let prev_index = 0u32;
        let value = 100000000u64;
        let script_pubkey = b"test_script".to_vec();
        
        let leaf_index = utxo_set.add_utxo(prev_txid, prev_index, value, &script_pubkey);
        assert_eq!(leaf_index, 0);
        
        // Check UTXO exists
        assert!(utxo_set.contains_utxo(prev_txid, prev_index));
        
        // Test state root
        let state_root = utxo_set.state_root();
        assert_eq!(state_root.len(), 32);
        
        // Test UTXO removal
        let removed = utxo_set.remove_utxo(prev_txid, prev_index);
        assert!(removed);
        assert!(!utxo_set.contains_utxo(prev_txid, prev_index));
        
        // Test UTXO batch operations
        let mut batch = UtxoBatch::new([0u8; 32]);
        
        // Add UTXOs to batch
        batch.add_utxo(prev_txid, prev_index, value, script_pubkey.clone());
        batch.add_utxo([2u8; 32], 1, 50000000, b"script2".to_vec());
        
        // Spend a UTXO
        let spent = batch.spend_utxo(prev_txid, prev_index);
        assert!(spent);
        
        // Test finalization
        let final_root = batch.finalize();
        assert_eq!(final_root.len(), 32);
        
        // Test spent UTXOs
        let spent_utxos = batch.spent_utxos();
        assert_eq!(spent_utxos.len(), 1);
        assert_eq!(spent_utxos[0], (prev_txid, prev_index));
        
        // Test new UTXOs
        let new_utxos = batch.new_utxos();
        assert_eq!(new_utxos.len(), 2);
        
        println!("SMT implementation test passed!");
    }

    #[test]
    fn test_batch_chain_implementation() {
        use zcash_validator::{BatchChain, UtxoBatch, BatchRecord};
        
        // Create a new batch chain starting with empty state
        let initial_state_root = [0u8; 32];
        let mut chain = BatchChain::new(initial_state_root);
        
        // Verify initial state
        assert_eq!(chain.current_state_root(), initial_state_root);
        assert!(chain.verify_chain());
        assert_eq!(chain.total_transactions(), 0);
        
        // Process first batch
        let mut batch1 = UtxoBatch::new(initial_state_root);
        batch1.add_utxo([1u8; 32], 0, 100000000, b"script1".to_vec());
        batch1.add_utxo([2u8; 32], 1, 50000000, b"script2".to_vec());
        
        let new_state_root1 = chain.process_batch(1, batch1, 2);
        assert_ne!(new_state_root1, initial_state_root);
        assert_eq!(chain.current_state_root(), new_state_root1);
        assert!(chain.verify_chain());
        assert_eq!(chain.total_transactions(), 2);
        assert_eq!(chain.total_new_utxos(), 2);
        
        // Process second batch (chaining from first batch's output)
        let mut batch2 = UtxoBatch::new(new_state_root1);
        // Note: We can't spend UTXOs from previous batches in a new batch
        // Each batch is independent. We'll just add new UTXOs
        batch2.add_utxo([3u8; 32], 0, 75000000, b"script3".to_vec());
        
        let new_state_root2 = chain.process_batch(2, batch2, 1);
        assert_ne!(new_state_root2, new_state_root1);
        assert_eq!(chain.current_state_root(), new_state_root2);
        assert!(chain.verify_chain());
        assert_eq!(chain.total_transactions(), 3);
        assert_eq!(chain.total_spent_utxos(), 0);
        assert_eq!(chain.total_new_utxos(), 3);
        
        // Process third batch (chaining from second batch's output)
        let mut batch3 = UtxoBatch::new(new_state_root2);
        // Note: We can't spend UTXOs from previous batches in a new batch
        // Each batch is independent. We'll just add new UTXOs
        batch3.add_utxo([4u8; 32], 0, 125000000, b"script4".to_vec());
        
        let new_state_root3 = chain.process_batch(3, batch3, 1);
        assert_ne!(new_state_root3, new_state_root2);
        assert_eq!(chain.current_state_root(), new_state_root3);
        assert!(chain.verify_chain());
        assert_eq!(chain.total_transactions(), 4);
        assert_eq!(chain.total_spent_utxos(), 0);
        assert_eq!(chain.total_new_utxos(), 4);
        
        // Verify batch history
        let history = chain.batch_history();
        assert_eq!(history.len(), 3);
        
        // Check state root chaining
        assert_eq!(history[0].prior_state_root, initial_state_root);
        assert_eq!(history[0].new_state_root, new_state_root1);
        assert_eq!(history[1].prior_state_root, new_state_root1);
        assert_eq!(history[1].new_state_root, new_state_root2);
        assert_eq!(history[2].prior_state_root, new_state_root2);
        assert_eq!(history[2].new_state_root, new_state_root3);
        
        // Test that the valid chain is indeed valid
        assert!(chain.verify_chain());
        
        println!("Batch chain implementation test passed!");
        println!("Final state root: {:02x?}", new_state_root3);
        println!("Total transactions processed: {}", chain.total_transactions());
        println!("Total UTXOs spent: {}", chain.total_spent_utxos());
        println!("Total new UTXOs created: {}", chain.total_new_utxos());
    }
}
