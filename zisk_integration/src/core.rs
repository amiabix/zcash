//! Core data structures and types for ZisK-Zcash validation

use alloc::vec::Vec;
use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Zcash transaction version
pub type TransactionVersion = u32;

/// Zcash version group ID
pub type VersionGroupId = u32;

/// Lock time value
pub type LockTime = u32;

/// Expiry height
pub type ExpiryHeight = u32;

/// Value in zatoshis (smallest Zcash unit)
pub type Zatoshis = u64;

/// Sequence number
pub type Sequence = u32;

/// Block height
pub type BlockHeight = u32;

/// Transaction hash (32 bytes)
pub type TxHash = [u8; 32];

/// Script hash (32 bytes)
pub type ScriptHash = [u8; 32];

/// Public key (32 bytes)
pub type PublicKey = [u8; 32];

/// Signature (64 bytes) - using Vec<u8> for serde compatibility
pub type Signature = Vec<u8>;

/// Commitment (32 bytes)
pub type Commitment = [u8; 32];

/// Nullifier (32 bytes)
pub type Nullifier = [u8; 32];

/// Anchor (32 bytes)
pub type Anchor = [u8; 32];

/// Ephemeral key (32 bytes)
pub type EphemeralKey = [u8; 32];

/// Merkle root (32 bytes)
pub type MerkleRoot = [u8; 32];

/// Zcash transaction structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZcashTransaction {
    /// Transaction version
    pub version: TransactionVersion,
    /// Version group ID
    pub version_group_id: VersionGroupId,
    /// Lock time
    pub lock_time: LockTime,
    /// Expiry height
    pub expiry_height: ExpiryHeight,
    /// Transparent inputs
    pub transparent_inputs: Vec<TransparentInput>,
    /// Transparent outputs
    pub transparent_outputs: Vec<TransparentOutput>,
    /// Sapling bundle (if present)
    pub sapling_bundle: Option<SaplingBundle>,
    /// Orchard bundle (if present)
    pub orchard_bundle: Option<OrchardBundle>,
}

/// Transparent transaction input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransparentInput {
    /// Previous output hash
    pub prevout_hash: TxHash,
    /// Previous output index
    pub prevout_index: u32,
    /// Script signature
    pub script_sig: Vec<u8>,
    /// Sequence number
    pub sequence: Sequence,
}

/// Transparent transaction output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransparentOutput {
    /// Output value in zatoshis
    pub value: Zatoshis,
    /// Script public key
    pub script_pubkey: Vec<u8>,
}

/// Sapling bundle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaplingBundle {
    /// Value balance
    pub value_balance: i64,
    /// Sapling spends
    pub spends: Vec<SaplingSpend>,
    /// Sapling outputs
    pub outputs: Vec<SaplingOutput>,
    /// Binding signature
    pub binding_signature: Signature,
}

/// Sapling spend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaplingSpend {
    /// Nullifier
    pub nullifier: Nullifier,
    /// Value commitment
    pub cv: Commitment,
    /// Anchor
    pub anchor: Anchor,
    /// Proof
    pub proof: Vec<u8>,
    /// Spend description
    pub spend_description: Vec<u8>,
}

/// Sapling output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaplingOutput {
    /// Note commitment
    pub cmu: Commitment,
    /// Value commitment
    pub cv: Commitment,
    /// Ephemeral public key
    pub ephemeral_key: EphemeralKey,
    /// Encrypted ciphertext
    pub enc_ciphertext: Vec<u8>,
    /// Out ciphertext
    pub out_ciphertext: Vec<u8>,
    /// Proof
    pub proof: Vec<u8>,
}

/// Orchard bundle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchardBundle {
    /// Actions
    pub actions: Vec<OrchardAction>,
    /// Value commitment
    pub value_commitment: Commitment,
    /// Binding signature
    pub binding_signature: Signature,
}

/// Orchard action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchardAction {
    /// Nullifier
    pub nullifier: Nullifier,
    /// Value commitment
    pub cv: Commitment,
    /// Note commitment
    pub cmu: Commitment,
    /// Ephemeral key
    pub ephemeral_key: EphemeralKey,
    /// Encrypted ciphertext
    pub enc_ciphertext: Vec<u8>,
    /// Out ciphertext
    pub out_ciphertext: Vec<u8>,
    /// Proof
    pub proof: Vec<u8>,
}

/// Validation configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Maximum transaction size in bytes
    pub max_tx_size: usize,
    /// Maximum number of inputs
    pub max_inputs: usize,
    /// Maximum number of outputs
    pub max_outputs: usize,
    /// Maximum fee in zatoshis
    pub max_fee: Zatoshis,
    /// Minimum fee in zatoshis
    pub min_fee: Zatoshis,
    /// Maximum value in zatoshis
    pub max_value: Zatoshis,
    /// Enable strict validation
    pub strict_mode: bool,
    /// Enable batch processing
    pub enable_batch: bool,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Proof generation timeout in seconds
    pub proof_timeout: u64,
    /// Memory limit for proof generation in MB
    pub memory_limit: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_tx_size: 100_000, // 100KB
            max_inputs: 1000,
            max_outputs: 1000,
            max_fee: 1_000_000, // 0.01 ZEC
            min_fee: 1_000, // 0.00001 ZEC
            max_value: 21_000_000 * 100_000_000, // 21M ZEC
            strict_mode: true,
            enable_batch: true,
            max_batch_size: 100,
            proof_timeout: 300, // 5 minutes
            memory_limit: 8192, // 8GB
        }
    }
}

/// Single transaction validation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the transaction is valid
    pub is_valid: bool,
    /// Total input value in zatoshis
    pub total_input_value: Zatoshis,
    /// Total output value in zatoshis
    pub total_output_value: Zatoshis,
    /// Transaction fee in zatoshis
    pub fee: Zatoshis,
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
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation errors
    pub errors: Vec<String>,
}

/// Batch validation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchValidationResult {
    /// Total number of transactions
    pub total_transactions: usize,
    /// Number of valid transactions
    pub valid_transactions: usize,
    /// Number of invalid transactions
    pub invalid_transactions: usize,
    /// Whether the entire batch is valid
    pub batch_valid: bool,
    /// Total input value across all transactions
    pub total_input_value: Zatoshis,
    /// Total output value across all transactions
    pub total_output_value: Zatoshis,
    /// Total fee across all transactions
    pub total_fee: Zatoshis,
    /// Individual transaction results
    pub transaction_results: Vec<ValidationResult>,
    /// Batch-level warnings
    pub warnings: Vec<String>,
    /// Batch-level errors
    pub errors: Vec<String>,
}

/// STARK proof structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarkProof {
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Compressed proof data
    pub compressed_proof_data: Vec<u8>,
    /// Public inputs
    pub public_inputs: Vec<u64>,
    /// Proof metadata
    pub metadata: ProofMetadata,
}

/// Proof metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofMetadata {
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

/// Performance metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Parsing time in microseconds
    pub parsing_time_us: u64,
    /// Validation time in microseconds
    pub validation_time_us: u64,
    /// Proof generation time in microseconds
    pub proof_time_us: u64,
    /// Total execution time in microseconds
    pub total_time_us: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// RISC-V cycles executed
    pub riscv_cycles: u64,
    /// Proof size in bytes
    pub proof_size_bytes: usize,
}