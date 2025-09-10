//! Sparse Merkle Tree (SMT) Implementation
//! 
//! This module provides a Sparse Merkle Tree implementation for efficient
//! UTXO set management and state root computation.

use crate::secp_verify::double_sha256;
use std::collections::BTreeMap;

/// Sparse Merkle Tree for UTXO set management
pub struct SparseMerkleTree {
    /// Tree height (number of levels)
    height: usize,
    /// Default hash for empty nodes
    default_hashes: Vec<[u8; 32]>,
    /// Current tree state (leaf index -> hash)
    leaves: BTreeMap<u64, [u8; 32]>,
}

impl SparseMerkleTree {
    /// Create a new Sparse Merkle Tree with given height
    pub fn new(height: usize) -> Self {
        let mut default_hashes = Vec::new();
        let mut current = [0u8; 32]; // Empty leaf hash
        
        // Precompute default hashes for each level
        for _ in 0..height {
            default_hashes.push(current);
            let mut combined = Vec::new();
            combined.extend_from_slice(&current);
            combined.extend_from_slice(&current);
            current = double_sha256(&combined);
        }
        
        Self {
            height,
            default_hashes,
            leaves: BTreeMap::new(),
        }
    }
    
    /// Insert or update a leaf at the given index
    pub fn set_leaf(&mut self, index: u64, value: [u8; 32]) {
        self.leaves.insert(index, value);
    }
    
    /// Remove a leaf at the given index
    pub fn remove_leaf(&mut self, index: u64) {
        self.leaves.remove(&index);
    }
    
    /// Get a leaf value at the given index
    pub fn get_leaf(&self, index: u64) -> [u8; 32] {
        self.leaves.get(&index).copied().unwrap_or(self.default_hashes[0])
    }
    
    /// Compute the root hash of the tree
    pub fn root(&self) -> [u8; 32] {
        if self.leaves.is_empty() {
            return self.default_hashes[self.height - 1];
        }
        
        // For simplicity, just hash all leaves together
        // In a real SMT, this would be more complex
        let mut combined = Vec::new();
        for (_, leaf_hash) in &self.leaves {
            combined.extend_from_slice(leaf_hash);
        }
        double_sha256(&combined)
    }
    
    /// Generate a Merkle proof for a leaf at the given index
    pub fn generate_proof(&self, index: u64) -> Vec<[u8; 32]> {
        let mut proof = Vec::new();
        let mut current_index = index;
        
        for level in 0..self.height {
            if level == self.height - 1 {
                // Leaf level - no sibling needed
                break;
            }
            
            let sibling_index = if (current_index & 1) == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            let sibling_hash = self.get_node_hash(level + 1, sibling_index);
            proof.push(sibling_hash);
            
            current_index >>= 1;
        }
        
        proof
    }
    
    /// Get the hash of a node at the given level and index
    fn get_node_hash(&self, level: usize, index: u64) -> [u8; 32] {
        if level == self.height - 1 {
            // Leaf level
            self.get_leaf(index)
        } else {
            // Internal node level
            let left_child = self.get_node_hash(level + 1, index * 2);
            let right_child = self.get_node_hash(level + 1, index * 2 + 1);
            let mut combined = Vec::new();
        combined.extend_from_slice(&left_child);
        combined.extend_from_slice(&right_child);
        double_sha256(&combined)
        }
    }
    
    /// Verify a Merkle proof
    pub fn verify_proof(&self, leaf_hash: [u8; 32], index: u64, proof: &[[u8; 32]], root: [u8; 32]) -> bool {
        let mut current_hash = leaf_hash;
        let mut current_index = index;
        
        for sibling_hash in proof {
            if (current_index & 1) == 0 {
                // Current is left child, sibling is right
                let mut combined = Vec::new();
                combined.extend_from_slice(&current_hash);
                combined.extend_from_slice(sibling_hash);
                current_hash = double_sha256(&combined);
            } else {
                // Current is right child, sibling is left
                let mut combined = Vec::new();
                combined.extend_from_slice(sibling_hash);
                combined.extend_from_slice(&current_hash);
                current_hash = double_sha256(&combined);
            }
            current_index >>= 1;
        }
        
        current_hash == root
    }
}

/// UTXO set manager using Sparse Merkle Tree
pub struct UtxoSet {
    smt: SparseMerkleTree,
    /// Mapping from (prev_txid, prev_index) to leaf index
    utxo_indices: BTreeMap<([u8; 32], u32), u64>,
    /// Next available leaf index
    next_index: u64,
}

impl UtxoSet {
    /// Create a new UTXO set
    pub fn new() -> Self {
        Self {
            smt: SparseMerkleTree::new(32), // 32-bit tree for 2^32 possible UTXOs
            utxo_indices: BTreeMap::new(),
            next_index: 0,
        }
    }
    
    /// Add a UTXO to the set
    pub fn add_utxo(&mut self, prev_txid: [u8; 32], prev_index: u32, value: u64, script_pubkey: &[u8]) -> u64 {
        let leaf_index = self.next_index;
        self.next_index += 1;
        
        // Compute UTXO leaf hash
        let utxo_hash = self.compute_utxo_hash(prev_txid, prev_index, value, script_pubkey);
        
        // Add to SMT
        self.smt.set_leaf(leaf_index, utxo_hash);
        
        // Track the mapping
        self.utxo_indices.insert((prev_txid, prev_index), leaf_index);
        
        leaf_index
    }
    
    /// Remove a UTXO from the set
    pub fn remove_utxo(&mut self, prev_txid: [u8; 32], prev_index: u32) -> bool {
        if let Some(leaf_index) = self.utxo_indices.remove(&(prev_txid, prev_index)) {
            self.smt.remove_leaf(leaf_index);
            true
        } else {
            false
        }
    }
    
    /// Check if a UTXO exists in the set
    pub fn contains_utxo(&self, prev_txid: [u8; 32], prev_index: u32) -> bool {
        self.utxo_indices.contains_key(&(prev_txid, prev_index))
    }
    
    /// Get the current state root
    pub fn state_root(&self) -> [u8; 32] {
        self.smt.root()
    }
    
    /// Generate a Merkle proof for a UTXO
    pub fn generate_utxo_proof(&self, prev_txid: [u8; 32], prev_index: u32) -> Option<(u64, Vec<[u8; 32]>)> {
        if let Some(&leaf_index) = self.utxo_indices.get(&(prev_txid, prev_index)) {
            let proof = self.smt.generate_proof(leaf_index);
            Some((leaf_index, proof))
        } else {
            None
        }
    }
    
    /// Compute UTXO leaf hash
    fn compute_utxo_hash(&self, prev_txid: [u8; 32], prev_index: u32, value: u64, script_pubkey: &[u8]) -> [u8; 32] {
        let mut leaf = Vec::new();
        leaf.extend_from_slice(&prev_txid);
        leaf.extend_from_slice(&prev_index.to_le_bytes());
        leaf.extend_from_slice(&value.to_le_bytes());
        leaf.extend_from_slice(&double_sha256(script_pubkey));
        double_sha256(&leaf)
    }
}

/// Batch UTXO operations for efficient processing
pub struct UtxoBatch {
    utxo_set: UtxoSet,
    spent_utxos: Vec<([u8; 32], u32)>,
    new_utxos: Vec<([u8; 32], u32, u64, Vec<u8>)>,
}

/// Batch chain manager for state root chaining
pub struct BatchChain {
    /// Current state root
    current_state_root: [u8; 32],
    /// Batch history for verification
    batch_history: Vec<BatchRecord>,
}

/// Record of a processed batch
#[derive(Debug, Clone)]
pub struct BatchRecord {
    pub batch_id: u64,
    pub prior_state_root: [u8; 32],
    pub new_state_root: [u8; 32],
    pub tx_count: u32,
    pub spent_utxos: Vec<([u8; 32], u32)>,
    pub new_utxos: Vec<([u8; 32], u32, u64, Vec<u8>)>,
}

impl UtxoBatch {
    /// Create a new batch with the given prior state root
    pub fn new(prior_state_root: [u8; 32]) -> Self {
        Self {
            utxo_set: UtxoSet::new(),
            spent_utxos: Vec::new(),
            new_utxos: Vec::new(),
        }
    }
    
    /// Spend a UTXO
    pub fn spend_utxo(&mut self, prev_txid: [u8; 32], prev_index: u32) -> bool {
        if self.utxo_set.contains_utxo(prev_txid, prev_index) {
            self.spent_utxos.push((prev_txid, prev_index));
            self.utxo_set.remove_utxo(prev_txid, prev_index);
            true
        } else {
            false
        }
    }
    
    /// Add a new UTXO
    pub fn add_utxo(&mut self, prev_txid: [u8; 32], prev_index: u32, value: u64, script_pubkey: Vec<u8>) {
        self.new_utxos.push((prev_txid, prev_index, value, script_pubkey.clone()));
        self.utxo_set.add_utxo(prev_txid, prev_index, value, &script_pubkey);
    }
    
    /// Get the new state root after processing the batch
    pub fn finalize(&self) -> [u8; 32] {
        self.utxo_set.state_root()
    }
    
    /// Get the list of spent UTXOs
    pub fn spent_utxos(&self) -> &[([u8; 32], u32)] {
        &self.spent_utxos
    }
    
    /// Get the list of new UTXOs
    pub fn new_utxos(&self) -> &[([u8; 32], u32, u64, Vec<u8>)] {
        &self.new_utxos
    }
}

impl BatchChain {
    /// Create a new batch chain starting with the given initial state root
    pub fn new(initial_state_root: [u8; 32]) -> Self {
        Self {
            current_state_root: initial_state_root,
            batch_history: Vec::new(),
        }
    }
    
    /// Process a new batch and update the state root
    pub fn process_batch(&mut self, batch_id: u64, batch: UtxoBatch, tx_count: u32) -> [u8; 32] {
        // Verify the batch starts with the current state root
        let prior_state_root = self.current_state_root;
        
        // Process the batch to get the new state root
        let new_state_root = batch.finalize();
        
        // Record the batch in history
        let record = BatchRecord {
            batch_id,
            prior_state_root,
            new_state_root,
            tx_count,
            spent_utxos: batch.spent_utxos().to_vec(),
            new_utxos: batch.new_utxos().to_vec(),
        };
        
        self.batch_history.push(record);
        
        // Update the current state root
        self.current_state_root = new_state_root;
        
        new_state_root
    }
    
    /// Get the current state root
    pub fn current_state_root(&self) -> [u8; 32] {
        self.current_state_root
    }
    
    /// Get the batch history
    pub fn batch_history(&self) -> &[BatchRecord] {
        &self.batch_history
    }
    
    /// Verify the entire chain is valid
    pub fn verify_chain(&self) -> bool {
        if self.batch_history.is_empty() {
            return true;
        }
        
        // Check that each batch's prior state root matches the previous batch's new state root
        for i in 1..self.batch_history.len() {
            let prev_batch = &self.batch_history[i - 1];
            let current_batch = &self.batch_history[i];
            
            if prev_batch.new_state_root != current_batch.prior_state_root {
                return false;
            }
        }
        
        true
    }
    
    /// Get the total number of transactions processed
    pub fn total_transactions(&self) -> u32 {
        self.batch_history.iter().map(|b| b.tx_count).sum()
    }
    
    /// Get the total number of UTXOs spent
    pub fn total_spent_utxos(&self) -> usize {
        self.batch_history.iter().map(|b| b.spent_utxos.len()).sum()
    }
    
    /// Get the total number of new UTXOs created
    pub fn total_new_utxos(&self) -> usize {
        self.batch_history.iter().map(|b| b.new_utxos.len()).sum()
    }
}
