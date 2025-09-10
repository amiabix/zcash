//! RPC Types and Request/Response structures

use serde::{Deserialize, Serialize};
use crate::core::*;
use crate::error::*;

/// RPC Request for single transaction validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTransactionRequest {
    /// Raw transaction bytes (hex encoded)
    pub tx_bytes: String,
    /// UTXO data with Merkle proofs
    pub utxos: Vec<UtxoRequest>,
    /// Prior state root (hex encoded)
    pub prior_state_root: String,
    /// Block height
    pub block_height: u32,
    /// Consensus branch ID
    pub consensus_branch_id: u32,
}

/// RPC Request for batch transaction validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateBatchRequest {
    /// Raw transaction batch bytes (hex encoded)
    pub batch_bytes: String,
    /// UTXO data with Merkle proofs
    pub utxos: Vec<UtxoRequest>,
    /// Prior state root (hex encoded)
    pub prior_state_root: String,
    /// Block height
    pub block_height: u32,
    /// Consensus branch ID
    pub consensus_branch_id: u32,
}

/// UTXO data for RPC requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRequest {
    /// Previous transaction ID (hex encoded)
    pub prev_txid: String,
    /// Previous output index
    pub prev_index: u32,
    /// Value in zatoshis
    pub value: u64,
    /// Script public key (hex encoded)
    pub script_pubkey: String,
    /// Merkle proof (hex encoded)
    pub merkle_proof: Vec<String>,
    /// Leaf index in Merkle tree
    pub leaf_index: u64,
    /// Block height when UTXO was created
    pub height: u32,
    /// Whether UTXO is spendable
    pub spendable: bool,
}

/// RPC Response for transaction validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTransactionResponse {
    /// Whether validation was successful
    pub success: bool,
    /// Validation result
    pub result: Option<ValidationResultResponse>,
    /// Error message if validation failed
    pub error: Option<String>,
    /// Processing time in microseconds
    pub processing_time_us: u64,
}

/// RPC Response for batch validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateBatchResponse {
    /// Whether validation was successful
    pub success: bool,
    /// Batch validation result
    pub result: Option<BatchValidationResultResponse>,
    /// Error message if validation failed
    pub error: Option<String>,
    /// Processing time in microseconds
    pub processing_time_us: u64,
}

/// Validation result for RPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultResponse {
    /// Whether the transaction is valid
    pub is_valid: bool,
    /// Total input value in zatoshis
    pub total_input_value: u64,
    /// Total output value in zatoshis
    pub total_output_value: u64,
    /// Transaction fee in zatoshis
    pub fee: u64,
    /// Transparent balance change
    pub transparent_balance: i64,
    /// Sapling balance change
    pub sapling_balance: i64,
    /// Orchard balance change
    pub orchard_balance: i64,
    /// Whether nullifiers are valid
    pub nullifiers_valid: bool,
    /// Whether commitments are valid
    pub commitments_valid: bool,
    /// Whether signatures are valid
    pub signatures_valid: bool,
    /// Whether zk-SNARK proofs are valid
    pub zk_proofs_valid: bool,
    /// New state root after processing (hex encoded)
    pub new_state_root: String,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation errors
    pub errors: Vec<String>,
}

/// Batch validation result for RPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationResultResponse {
    /// Total number of transactions
    pub total_transactions: usize,
    /// Number of valid transactions
    pub valid_transactions: usize,
    /// Number of invalid transactions
    pub invalid_transactions: usize,
    /// Whether the entire batch is valid
    pub batch_valid: bool,
    /// Total input value across all transactions
    pub total_input_value: u64,
    /// Total output value across all transactions
    pub total_output_value: u64,
    /// Total fee across all transactions
    pub total_fee: u64,
    /// New state root after processing (hex encoded)
    pub new_state_root: String,
    /// Individual transaction results
    pub transaction_results: Vec<ValidationResultResponse>,
    /// Batch-level warnings
    pub warnings: Vec<String>,
    /// Batch-level errors
    pub errors: Vec<String>,
}

/// RPC Request for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofRequest {
    /// Validation result to generate proof for
    pub validation_result: ValidationResultResponse,
    /// Transaction data (hex encoded)
    pub tx_data: String,
    /// UTXO data used in validation
    pub utxos: Vec<UtxoRequest>,
}

/// RPC Response for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofResponse {
    /// Whether proof generation was successful
    pub success: bool,
    /// STARK proof data
    pub proof: Option<StarkProofResponse>,
    /// Error message if proof generation failed
    pub error: Option<String>,
    /// Proof generation time in microseconds
    pub generation_time_us: u64,
}

/// STARK proof for RPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkProofResponse {
    /// Proof data (hex encoded)
    pub proof_data: String,
    /// Compressed proof data (hex encoded)
    pub compressed_proof_data: String,
    /// Public inputs
    pub public_inputs: Vec<u64>,
    /// Proof metadata
    pub metadata: ProofMetadataResponse,
}

/// Proof metadata for RPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadataResponse {
    /// Proof ID
    pub proof_id: String,
    /// Generation timestamp
    pub timestamp: u64,
    /// RISC-V cycles executed
    pub riscv_cycles: u64,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Proof size in bytes
    pub proof_size: usize,
    /// Compressed proof size in bytes
    pub compressed_size: usize,
    /// Generation time in microseconds
    pub generation_time_us: u64,
    /// Verification time in microseconds
    pub verification_time_us: u64,
}

/// RPC Request for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    /// STARK proof to verify
    pub proof: StarkProofResponse,
    /// Expected public inputs
    pub expected_public_inputs: Vec<u64>,
}

/// RPC Response for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofResponse {
    /// Whether verification was successful
    pub success: bool,
    /// Whether the proof is valid
    pub is_valid: bool,
    /// Error message if verification failed
    pub error: Option<String>,
    /// Verification time in microseconds
    pub verification_time_us: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Number of processed transactions
    pub processed_transactions: u64,
    /// Number of generated proofs
    pub generated_proofs: u64,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}
