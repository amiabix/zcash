//! UTXO Validation Module
//! 
//! This module provides UTXO validation functionality with Merkle proofs.

use crate::secp_verify::double_sha256;
use sha2::{Sha256, Digest};

/// UTXO entry with Merkle proof for validation
#[derive(Debug, Clone)]
pub struct UtxoEntry {
    pub prev_txid: [u8; 32],
    pub prev_index: u32,
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub merkle_proof: Vec<[u8; 32]>, // Merkle path from leaf to root
    pub leaf_index: u64, // Position in the Merkle tree
}

/// Verify that a UTXO exists in the prior state root using Merkle proof
pub fn verify_utxo_inclusion(utxo: &UtxoEntry, prior_state_root: &[u8; 32]) -> bool {
    // Compute the UTXO leaf hash
    let leaf_hash = utxo_leaf_double_sha(
        &utxo.prev_txid,
        utxo.prev_index,
        utxo.value,
        &utxo.script_pubkey
    );
    
    // Verify the Merkle proof
    let computed_root = verify_merkle_branch(
        &leaf_hash,
        utxo.leaf_index,
        &utxo.merkle_proof
    );
    
    // Check that the computed root matches the prior state root
    computed_root == *prior_state_root
}

/// Check for double-spending by ensuring UTXOs are not reused
pub fn check_double_spend(utxos: &[UtxoEntry]) -> bool {
    // For transparent UTXOs, check that no (prev_txid, prev_index) pair is duplicated
    let mut seen = Vec::new();
    for utxo in utxos {
        let key = (utxo.prev_txid, utxo.prev_index);
        if seen.contains(&key) {
            return false; // Double spend detected
        }
        seen.push(key);
    }
    true
}

/// UTXO leaf serialization + double-SHA256 hash
fn utxo_leaf_double_sha(prev_txid: &[u8;32], prev_index: u32, value: u64, script_pubkey: &[u8]) -> [u8;32] {
    // leaf_bytes: prev_txid(32) || prev_index(u32 LE) || value(u64 LE) || SHA256(script_pubkey)
    let mut leaf: Vec<u8> = Vec::new();
    leaf.extend_from_slice(prev_txid);
    leaf.extend_from_slice(&prev_index.to_le_bytes());
    leaf.extend_from_slice(&value.to_le_bytes());
    leaf.extend_from_slice(&double_sha256(script_pubkey));
    double_sha256(&leaf)
}

/// Merkle branch verification (double-sha, left||right concatenation)
fn verify_merkle_branch(leaf_hash: &[u8;32], mut index: u64, path: &Vec<[u8;32]>) -> [u8;32] {
    let mut cur = *leaf_hash;
    let mut hasher = sha2::Sha256::new();

    for sib in path.iter() {
        // if index LSB == 0 -> current is left, sibling right: H(curr || sib)
        // else -> current is right, sibling left: H(sib || curr)
        if (index & 1) == 0 {
            // cur || sib
            hasher.update(&cur);
            hasher.update(sib);
        } else {
            hasher.update(sib);
            hasher.update(&cur);
        }
        let first = hasher.finalize_reset();
        hasher.update(&first);
        let second = hasher.finalize_reset();
        cur.copy_from_slice(&second);
        index >>= 1;
    }
    cur
}
