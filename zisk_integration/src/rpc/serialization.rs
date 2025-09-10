//! Serialization utilities for RPC requests and responses

use crate::core::*;
use crate::error::*;
use crate::rpc::types::*;
use hex;
use serde_json;

/// Convert hex string to bytes
pub fn hex_to_bytes(hex_str: &str) -> ZcashResult<Vec<u8>> {
    hex::decode(hex_str.trim_start_matches("0x"))
        .map_err(|e| ZcashValidationError::SystemError(SystemError::ParseError(format!("Invalid hex: {:?}", e))))
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert hex string to 32-byte array
pub fn hex_to_txhash(hex_str: &str) -> ZcashResult<TxHash> {
    let bytes = hex_to_bytes(hex_str)?;
    if bytes.len() != 32 {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Invalid txid length".into())));
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

/// Convert 32-byte array to hex string
pub fn txhash_to_hex(hash: &TxHash) -> String {
    bytes_to_hex(hash)
}

/// Convert UtxoRequest to internal UtxoEntry
pub fn utxo_request_to_entry(utxo_req: &UtxoRequest) -> ZcashResult<UtxoEntry> {
    let prev_txid = hex_to_txhash(&utxo_req.prev_txid)?;
    let script_pubkey = hex_to_bytes(&utxo_req.script_pubkey)?;
    
    let mut merkle_proof = Vec::new();
    for proof_hex in &utxo_req.merkle_proof {
        let proof_hash = hex_to_txhash(proof_hex)?;
        merkle_proof.push(proof_hash);
    }
    
    Ok(UtxoEntry {
        prev_txid,
        prev_index: utxo_req.prev_index,
        value: utxo_req.value,
        script_pubkey,
        merkle_proof,
        leaf_index: utxo_req.leaf_index,
        height: utxo_req.height,
        spendable: utxo_req.spendable,
    })
}

/// Convert internal UtxoEntry to UtxoRequest
pub fn utxo_entry_to_request(utxo: &UtxoEntry) -> UtxoRequest {
    let mut merkle_proof = Vec::new();
    for proof_hash in &utxo.merkle_proof {
        merkle_proof.push(txhash_to_hex(proof_hash));
    }
    
    UtxoRequest {
        prev_txid: txhash_to_hex(&utxo.prev_txid),
        prev_index: utxo.prev_index,
        value: utxo.value,
        script_pubkey: bytes_to_hex(&utxo.script_pubkey),
        merkle_proof,
        leaf_index: utxo.leaf_index,
        height: utxo.height,
        spendable: utxo.spendable,
    }
}

/// Convert internal ValidationResult to RPC response
pub fn validation_result_to_response(result: &ValidationResult) -> ValidationResultResponse {
    ValidationResultResponse {
        is_valid: result.is_valid,
        total_input_value: result.total_input_value,
        total_output_value: result.total_output_value,
        fee: result.fee,
        transparent_balance: result.transparent_balance,
        sapling_balance: result.sapling_balance,
        orchard_balance: result.orchard_balance,
        nullifiers_valid: result.nullifiers_valid,
        commitments_valid: result.commitments_valid,
        signatures_valid: result.signatures_valid,
        zk_proofs_valid: result.zk_proofs_valid,
        new_state_root: txhash_to_hex(&result.new_state_root),
        warnings: result.warnings.clone(),
        errors: result.errors.clone(),
    }
}

/// Convert internal BatchValidationResult to RPC response
pub fn batch_validation_result_to_response(result: &BatchValidationResult) -> BatchValidationResultResponse {
    let transaction_results = result.transaction_results.iter()
        .map(validation_result_to_response)
        .collect();
    
    BatchValidationResultResponse {
        total_transactions: result.total_transactions,
        valid_transactions: result.valid_transactions,
        invalid_transactions: result.invalid_transactions,
        batch_valid: result.batch_valid,
        total_input_value: result.total_input_value,
        total_output_value: result.total_output_value,
        total_fee: result.total_fee,
        new_state_root: txhash_to_hex(&result.new_state_root),
        transaction_results,
        warnings: result.warnings.clone(),
        errors: result.errors.clone(),
    }
}

/// Convert internal StarkProof to RPC response
pub fn stark_proof_to_response(proof: &StarkProof) -> StarkProofResponse {
    StarkProofResponse {
        proof_data: bytes_to_hex(&proof.proof_data),
        compressed_proof_data: bytes_to_hex(&proof.compressed_proof_data),
        public_inputs: proof.public_inputs.clone(),
        metadata: proof_metadata_to_response(&proof.metadata),
    }
}

/// Convert internal ProofMetadata to RPC response
pub fn proof_metadata_to_response(metadata: &ProofMetadata) -> ProofMetadataResponse {
    ProofMetadataResponse {
        proof_id: metadata.proof_id.clone(),
        timestamp: metadata.timestamp,
        riscv_cycles: metadata.riscv_cycles,
        memory_usage: metadata.memory_usage,
        proof_size: metadata.proof_size,
        compressed_size: metadata.compressed_size,
        generation_time_us: metadata.generation_time_us,
        verification_time_us: metadata.verification_time_us,
    }
}

/// Parse transaction bytes from hex string
pub fn parse_transaction_bytes(tx_hex: &str) -> ZcashResult<Vec<u8>> {
    hex_to_bytes(tx_hex)
}

/// Parse batch input from hex string
pub fn parse_batch_bytes(batch_hex: &str) -> ZcashResult<Vec<u8>> {
    hex_to_bytes(batch_hex)
}

/// Serialize transaction bytes to hex string
pub fn serialize_transaction_bytes(tx_bytes: &[u8]) -> String {
    bytes_to_hex(tx_bytes)
}

/// Serialize batch input to hex string
pub fn serialize_batch_bytes(batch_bytes: &[u8]) -> String {
    bytes_to_hex(batch_bytes)
}

/// Validate hex string format
pub fn validate_hex_format(hex_str: &str) -> ZcashResult<()> {
    if hex_str.is_empty() {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Empty hex string".into())));
    }
    
    let cleaned = hex_str.trim_start_matches("0x");
    if cleaned.len() % 2 != 0 {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Invalid hex length".into())));
    }
    
    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Invalid hex characters".into())));
    }
    
    Ok(())
}

/// Validate transaction bytes format
pub fn validate_transaction_format(tx_bytes: &[u8]) -> ZcashResult<()> {
    if tx_bytes.is_empty() {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Empty transaction data".into())));
    }
    
    if tx_bytes.len() > 100_000 {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Transaction too large".into())));
    }
    
    Ok(())
}

/// Validate batch bytes format
pub fn validate_batch_format(batch_bytes: &[u8]) -> ZcashResult<()> {
    if batch_bytes.is_empty() {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Empty batch data".into())));
    }
    
    if batch_bytes.len() > 10_000_000 {
        return Err(ZcashValidationError::SystemError(SystemError::ParseError("Batch too large".into())));
    }
    
    Ok(())
}
