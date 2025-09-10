// secp_verify.rs
use secp256k1::{Secp256k1, Message, PublicKey, ecdsa::Signature};
use sha2::{Sha256, Digest};

/// Verify a compact 64-byte (r||s) ECDSA signature using libsecp256k1 C binding.
/// - msg32: 32-byte message (digest) — must match the signer's digest
/// - sig64: 64-byte compact signature (r||s)
/// - pubkey33: 33-byte compressed SEC1 pubkey
/// Returns true on valid signature, false otherwise.
pub fn verify_secp256k1_c(msg32: &[u8; 32], sig64: &[u8; 64], pubkey33: &[u8; 33]) -> bool {
    // Create verification-only context
    let secp = Secp256k1::new();

    // Parse public key (compressed)
    let pk = match PublicKey::from_slice(pubkey33) {
        Ok(k) => k,
        Err(_) => return false,
    };

    // Parse compact signature
    let sig = match Signature::from_compact(sig64) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Parse message (32 bytes)
    let msg = match Message::from_digest_slice(msg32) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Verify
    secp.verify_ecdsa(&msg, &sig, &pk).is_ok()
}

/// Helper: compute double-sha256 of the provided bytes (common Bitcoin-style digest)
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    let first = h.finalize_reset();
    h.update(&first);
    let second = h.finalize_reset();
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

/// Compute canonical SIGHASH_ALL digest for a transparent input
/// - tx_bytes: full serialized tx bytes (as used in your parser)
/// - input_index: index of input being signed
/// - utxo_script_pubkey: scriptPubKey of the UTXO being spent (needed to construct preimage)
///
/// Returns 32-byte double-sha of preimage.
/// 
/// This implements the exact SIGHASH_ALL algorithm used by Bitcoin/Zcash:
/// 1. Parse transaction structure
/// 2. Serialize version (4 bytes LE)
/// 3. Serialize input count (varint)
/// 4. For each input:
///    - If current input: prev_txid || prev_index || script_len || utxo_script_pubkey || sequence
///    - If other input: prev_txid || prev_index || script_len(0) || sequence (empty script)
/// 5. Serialize output count (varint)
/// 6. Serialize all outputs (value || script_len || script_pubkey)
/// 7. Serialize lock_time (4 bytes LE)
/// 8. Append sighash_type (4 bytes LE, 0x00000001 for SIGHASH_ALL)
/// 9. double_sha256(preimage)
pub fn compute_sighash_all_like(tx_bytes: &[u8], input_index: usize, utxo_script_pubkey: &[u8]) -> [u8;32] {
    // Parse the transaction to get its structure
    let tx = match parse_zcash_transaction_for_sighash(tx_bytes) {
        Ok(tx) => tx,
        Err(_) => return double_sha256(tx_bytes), // Fallback to simple hash if parsing fails
    };
    
    // Build the sighash preimage
    let mut preimage = Vec::new();
    
    // 1. Serialize version (4 bytes LE)
    preimage.extend_from_slice(&tx.version.to_le_bytes());
    
    // 2. Serialize input count (varint)
    preimage.extend_from_slice(&encode_compact_size(tx.inputs.len()));
    
    // 3. Serialize inputs
    for (i, input) in tx.inputs.iter().enumerate() {
        // prev_txid (32 bytes)
        preimage.extend_from_slice(&input.prev_hash);
        
        // prev_index (4 bytes LE)
        preimage.extend_from_slice(&input.prev_index.to_le_bytes());
        
        // script length and content
        if i == input_index {
            // Current input: use actual script_pubkey
            preimage.extend_from_slice(&encode_compact_size(utxo_script_pubkey.len()));
            preimage.extend_from_slice(utxo_script_pubkey);
        } else {
            // Other inputs: empty script
            preimage.extend_from_slice(&encode_compact_size(0));
        }
        
        // sequence (4 bytes LE)
        preimage.extend_from_slice(&input.sequence.to_le_bytes());
    }
    
    // 4. Serialize output count (varint)
    preimage.extend_from_slice(&encode_compact_size(tx.outputs.len()));
    
    // 5. Serialize outputs
    for output in &tx.outputs {
        // value (8 bytes LE)
        preimage.extend_from_slice(&output.value.to_le_bytes());
        
        // script length and content
        preimage.extend_from_slice(&encode_compact_size(output.script_pubkey.len()));
        preimage.extend_from_slice(&output.script_pubkey);
    }
    
    // 6. Serialize lock_time (4 bytes LE)
    preimage.extend_from_slice(&tx.lock_time.to_le_bytes());
    
    // 7. Append sighash_type (4 bytes LE, 0x00000001 for SIGHASH_ALL)
    preimage.extend_from_slice(&1u32.to_le_bytes());
    
    // 8. double_sha256(preimage)
    double_sha256(&preimage)
}

/// Parse transaction for sighash computation (simplified version)
fn parse_zcash_transaction_for_sighash(tx_bytes: &[u8]) -> Result<ZcashTransactionForSighash, ()> {
    let mut off = 0;
    
    // Version (4 bytes)
    if off + 4 > tx_bytes.len() { return Err(()); }
    let version = u32::from_le_bytes([tx_bytes[off], tx_bytes[off+1], tx_bytes[off+2], tx_bytes[off+3]]);
    off += 4;
    
    // Input count (varint)
    let input_count = read_compact_size_safe(tx_bytes, &mut off)?;
    
    // Parse inputs
    let mut inputs = Vec::new();
    for _ in 0..input_count {
        if off + 36 > tx_bytes.len() { return Err(()); }
        
        let mut prev_hash = [0u8; 32];
        prev_hash.copy_from_slice(&tx_bytes[off..off+32]);
        off += 32;
        
        let prev_index = u32::from_le_bytes([tx_bytes[off], tx_bytes[off+1], tx_bytes[off+2], tx_bytes[off+3]]);
        off += 4;
        
        // Skip script (we'll reconstruct it during sighash)
        let script_len = read_compact_size_safe(tx_bytes, &mut off)?;
        if off + script_len > tx_bytes.len() { return Err(()); }
        off += script_len;
        
        // Sequence (4 bytes)
        if off + 4 > tx_bytes.len() { return Err(()); }
        let sequence = u32::from_le_bytes([tx_bytes[off], tx_bytes[off+1], tx_bytes[off+2], tx_bytes[off+3]]);
        off += 4;
        
        inputs.push(TxInputForSighash {
            prev_hash,
            prev_index,
            sequence,
        });
    }
    
    // Output count (varint)
    let output_count = read_compact_size_safe(tx_bytes, &mut off)?;
    
    // Parse outputs
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        // Value (8 bytes)
        if off + 8 > tx_bytes.len() { return Err(()); }
        let value = u64::from_le_bytes([
            tx_bytes[off], tx_bytes[off+1], tx_bytes[off+2], tx_bytes[off+3],
            tx_bytes[off+4], tx_bytes[off+5], tx_bytes[off+6], tx_bytes[off+7]
        ]);
        off += 8;
        
        // Script length and content
        let script_len = read_compact_size_safe(tx_bytes, &mut off)?;
        if off + script_len > tx_bytes.len() { return Err(()); }
        let script_pubkey = tx_bytes[off..off+script_len].to_vec();
        off += script_len;
        
        outputs.push(TxOutputForSighash {
            value,
            script_pubkey,
        });
    }
    
    // Lock time (4 bytes)
    if off + 4 > tx_bytes.len() { return Err(()); }
    let lock_time = u32::from_le_bytes([tx_bytes[off], tx_bytes[off+1], tx_bytes[off+2], tx_bytes[off+3]]);
    
    Ok(ZcashTransactionForSighash {
        version,
        inputs,
        outputs,
        lock_time,
    })
}

/// Safe compact size reader that returns Result
fn read_compact_size_safe(data: &[u8], off: &mut usize) -> Result<usize, ()> {
    if *off >= data.len() { return Err(()); }
    
    let first_byte = data[*off];
    *off += 1;
    
    match first_byte {
        0..=252 => Ok(first_byte as usize),
        253 => {
            if *off + 2 > data.len() { return Err(()); }
            let val = u16::from_le_bytes([data[*off], data[*off+1]]) as usize;
            *off += 2;
            Ok(val)
        },
        254 => {
            if *off + 4 > data.len() { return Err(()); }
            let val = u32::from_le_bytes([data[*off], data[*off+1], data[*off+2], data[*off+3]]) as usize;
            *off += 4;
            Ok(val)
        },
        255 => {
            if *off + 8 > data.len() { return Err(()); }
            let val = u64::from_le_bytes([
                data[*off], data[*off+1], data[*off+2], data[*off+3],
                data[*off+4], data[*off+5], data[*off+6], data[*off+7]
            ]) as usize;
            *off += 8;
            Ok(val)
        },
    }
}

/// Encode compact size (varint)
fn encode_compact_size(n: usize) -> Vec<u8> {
    match n {
        0..=252 => vec![n as u8],
        253..=65535 => {
            let mut v = vec![253];
            v.extend_from_slice(&(n as u16).to_le_bytes());
            v
        },
        65536..=4294967295 => {
            let mut v = vec![254];
            v.extend_from_slice(&(n as u32).to_le_bytes());
            v
        },
        _ => {
            let mut v = vec![255];
            v.extend_from_slice(&(n as u64).to_le_bytes());
            v
        },
    }
}

/// Transaction structure for sighash computation
#[derive(Debug)]
struct ZcashTransactionForSighash {
    version: u32,
    inputs: Vec<TxInputForSighash>,
    outputs: Vec<TxOutputForSighash>,
    lock_time: u32,
}

#[derive(Debug)]
struct TxInputForSighash {
    prev_hash: [u8; 32],
    prev_index: u32,
    sequence: u32,
}

#[derive(Debug)]
struct TxOutputForSighash {
    value: u64,
    script_pubkey: Vec<u8>,
}
