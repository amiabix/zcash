//! Type conversions between parsing and core types

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};

use crate::core::*;
use crate::parsing::ZcashTransaction as ParsedTx;

/// Convert parsed transaction to complete transaction
pub fn convert_parsed_to_complete(parsed: ParsedTx) -> CompleteZcashTransaction {
    // Compute hash before moving parsed
    let tx_hash = compute_transaction_hash(&parsed);
    
    CompleteZcashTransaction {
        version: parsed.version,
        version_group_id: parsed.version_group_id,
        lock_time: parsed.lock_time,
        expiry_height: parsed.expiry_height,
        transparent_inputs: parsed.transparent_inputs.into_iter()
            .map(|i| TransparentInput {
                prevout_hash: i.prev_hash,
                prevout_index: i.prev_index,
                script_sig: i.script_sig,
                sequence: i.sequence,
            }).collect(),
        transparent_outputs: parsed.transparent_outputs.into_iter()
            .map(|o| TransparentOutput {
                value: o.value,
                script_pubkey: o.script_pubkey,
            }).collect(),
        sapling_bundle: parsed.sapling_bundle.map(|b| SaplingBundle {
            value_balance: b.value_balance,
            spends: b.spends.into_iter().map(|s| SaplingSpend {
                nullifier: s.nullifier,
                cv: s.cv,
                anchor: s.anchor,
                rk: s.rk,
                zkproof: s.proof,
                spend_auth_sig: s.spend_auth_sig,
            }).collect(),
            outputs: b.outputs.into_iter().map(|o| SaplingOutput {
                cmu: o.cmu,
                cv: o.cv,
                ephemeral_key: o.ephemeral_key,
                enc_ciphertext: o.enc_ciphertext,
                out_ciphertext: o.out_ciphertext,
                proof: o.zkproof,
            }).collect(),
            binding_signature: vec![0u8; 64], // Default value - would be parsed in real implementation
        }),
        orchard_bundle: parsed.orchard_bundle.map(|b| OrchardBundle {
            actions: b.actions.into_iter().map(|a| OrchardAction {
                nullifier: a.nullifier,
                cmu: a.cmu,
                cv: a.cv,
                ephemeral_key: a.rk, // Map rk to ephemeral_key
                enc_ciphertext: a.enc_ciphertext,
                out_ciphertext: a.out_ciphertext,
                proof: Vec::new(), // Default empty proof - would be parsed in real implementation
            }).collect(),
            value_commitment: [0u8; 32], // Default value - would be parsed in real implementation
            binding_signature: vec![0u8; 64], // Default value - would be parsed in real implementation
        }),
        tx_hash,
    }
}

/// Calculate transaction size for proper consumed bytes calculation
pub fn calculate_transaction_size(data: &[u8], version: u32) -> Result<usize, String> {
    if data.len() < 4 {
        return Err("Insufficient data for version".to_string());
    }
    
    let mut offset = 4; // Skip version
    
    // Version group ID (if v4+)
    if version >= 4 {
        if offset + 4 > data.len() {
            return Err("Insufficient data for version group ID".to_string());
        }
        offset += 4;
    }
    
    // Lock time
    if offset + 4 > data.len() {
        return Err("Insufficient data for lock time".to_string());
    }
    offset += 4;
    
    // Expiry height (if v4+)
    if version >= 4 {
        if offset + 4 > data.len() {
            return Err("Insufficient data for expiry height".to_string());
        }
        offset += 4;
    }
    
    // Transparent inputs
    if offset + 1 > data.len() {
        return Err("Insufficient data for input count".to_string());
    }
    let input_count = data[offset] as usize;
    offset += 1;
    
    for _ in 0..input_count {
        // prevout hash (32 bytes)
        if offset + 32 > data.len() {
            return Err("Insufficient data for prevout hash".to_string());
        }
        offset += 32;
        
        // prevout index (4 bytes)
        if offset + 4 > data.len() {
            return Err("Insufficient data for prevout index".to_string());
        }
        offset += 4;
        
        // scriptSig length + data
        if offset + 1 > data.len() {
            return Err("Insufficient data for scriptSig length".to_string());
        }
        let script_len = data[offset] as usize;
        offset += 1;
        
        if offset + script_len > data.len() {
            return Err("Insufficient data for scriptSig".to_string());
        }
        offset += script_len;
        
        // sequence (4 bytes)
        if offset + 4 > data.len() {
            return Err("Insufficient data for sequence".to_string());
        }
        offset += 4;
    }
    
    // Transparent outputs
    if offset + 1 > data.len() {
        return Err("Insufficient data for output count".to_string());
    }
    let output_count = data[offset] as usize;
    offset += 1;
    
    for _ in 0..output_count {
        // value (8 bytes)
        if offset + 8 > data.len() {
            return Err("Insufficient data for output value".to_string());
        }
        offset += 8;
        
        // scriptPubKey length + data
        if offset + 1 > data.len() {
            return Err("Insufficient data for scriptPubKey length".to_string());
        }
        let script_len = data[offset] as usize;
        offset += 1;
        
        if offset + script_len > data.len() {
            return Err("Insufficient data for scriptPubKey".to_string());
        }
        offset += script_len;
    }
    
    // Sapling bundle (if present)
    if version >= 4 {
        if offset + 1 > data.len() {
            return Err("Insufficient data for Sapling bundle flag".to_string());
        }
        let has_sapling = data[offset] != 0;
        offset += 1;
        
        if has_sapling {
            // Sapling value balance (8 bytes)
            if offset + 8 > data.len() {
                return Err("Insufficient data for Sapling value balance".to_string());
            }
            offset += 8;
            
            // Sapling spends count
            if offset + 1 > data.len() {
                return Err("Insufficient data for Sapling spends count".to_string());
            }
            let spend_count = data[offset] as usize;
            offset += 1;
            
            // Each spend: nullifier(32) + cv(32) + anchor(32) + rk(32) + proof(192) + spend_auth_sig(64)
            let spend_size = 32 + 32 + 32 + 32 + 192 + 64;
            if offset + spend_count * spend_size > data.len() {
                return Err("Insufficient data for Sapling spends".to_string());
            }
            offset += spend_count * spend_size;
            
            // Sapling outputs count
            if offset + 1 > data.len() {
                return Err("Insufficient data for Sapling outputs count".to_string());
            }
            let output_count = data[offset] as usize;
            offset += 1;
            
            // Each output: cmu(32) + cv(32) + ephemeral_key(32) + enc_ciphertext(580) + out_ciphertext(80) + proof(192)
            let output_size = 32 + 32 + 32 + 580 + 80 + 192;
            if offset + output_count * output_size > data.len() {
                return Err("Insufficient data for Sapling outputs".to_string());
            }
            offset += output_count * output_size;
        }
    }
    
    // Orchard bundle (if present)
    if version >= 5 {
        if offset + 1 > data.len() {
            return Err("Insufficient data for Orchard bundle flag".to_string());
        }
        let has_orchard = data[offset] != 0;
        offset += 1;
        
        if has_orchard {
            // Orchard flags (1 byte)
            if offset + 1 > data.len() {
                return Err("Insufficient data for Orchard flags".to_string());
            }
            offset += 1;
            
            // Orchard value balance (8 bytes)
            if offset + 8 > data.len() {
                return Err("Insufficient data for Orchard value balance".to_string());
            }
            offset += 8;
            
            // Orchard anchor (32 bytes)
            if offset + 32 > data.len() {
                return Err("Insufficient data for Orchard anchor".to_string());
            }
            offset += 32;
            
            // Orchard actions count
            if offset + 1 > data.len() {
                return Err("Insufficient data for Orchard actions count".to_string());
            }
            let action_count = data[offset] as usize;
            offset += 1;
            
            // Each action: nullifier(32) + cmu(32) + cv(32) + rk(32) + enc_ciphertext(580) + out_ciphertext(80)
            let action_size = 32 + 32 + 32 + 32 + 580 + 80;
            if offset + action_count * action_size > data.len() {
                return Err("Insufficient data for Orchard actions".to_string());
            }
            offset += action_count * action_size;
            
            // Orchard proof (variable size, typically 128 bytes)
            if offset + 128 > data.len() {
                return Err("Insufficient data for Orchard proof".to_string());
            }
            offset += 128;
        }
    }
    
    Ok(offset)
}

/// Compute transaction hash for the parsed transaction
fn compute_transaction_hash(parsed: &ParsedTx) -> [u8; 32] {
    // This is a simplified hash calculation
    // In production, this should compute the actual transaction hash
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(&parsed.version.to_le_bytes());
    if let Some(vgid) = parsed.version_group_id {
        hasher.update(&vgid.to_le_bytes());
    }
    hasher.update(&parsed.lock_time.to_le_bytes());
    if let Some(eh) = parsed.expiry_height {
        hasher.update(&eh.to_le_bytes());
    }
    
    // Hash inputs
    for input in &parsed.transparent_inputs {
        hasher.update(&input.prev_hash);
        hasher.update(&input.prev_index.to_le_bytes());
        hasher.update(&input.script_sig);
        hasher.update(&input.sequence.to_le_bytes());
    }
    
    // Hash outputs
    for output in &parsed.transparent_outputs {
        hasher.update(&output.value.to_le_bytes());
        hasher.update(&output.script_pubkey);
    }
    
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}
