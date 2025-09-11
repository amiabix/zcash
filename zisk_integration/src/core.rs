//! Core data structures and types for ZisK-Zcash validation

use alloc::vec::Vec;
use alloc::string::String;

#[cfg(feature = "host")]
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

/// Value commitment (32 bytes)
pub type ValueCommitment = [u8; 32];

/// State root (32 bytes)
pub type StateRoot = [u8; 32];

/// Merkle root (32 bytes)
pub type MerkleRoot = [u8; 32];

/// UTXO entry for validation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct UtxoEntry {
    pub value: Zatoshis,
    pub script_pubkey: Vec<u8>,
    pub height: BlockHeight,
    pub prev_txid: TxHash,
    pub prev_index: u32,
    pub merkle_proof: Vec<[u8; 32]>,
    pub leaf_index: u64,
    pub spendable: bool,
}

/// Transparent input
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct TransparentInput {
    pub prevout_hash: TxHash,
    pub prevout_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: Sequence,
}

/// Transparent output
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct TransparentOutput {
    pub value: Zatoshis,
    pub script_pubkey: Vec<u8>,
}

/// Sapling spend description
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct SaplingSpend {
    pub cv: ValueCommitment,
    pub anchor: Anchor,
    pub nullifier: Nullifier,
    pub rk: PublicKey,
    pub zkproof: Vec<u8>,
    pub spend_auth_sig: Signature,
}

/// Sapling output description
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct SaplingOutput {
    pub cv: ValueCommitment,
    pub cmu: Commitment,
    pub ephemeral_key: EphemeralKey,
    pub enc_ciphertext: Vec<u8>,
    pub out_ciphertext: Vec<u8>,
    pub proof: Vec<u8>,
}

/// Sapling bundle
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct SaplingBundle {
    pub value_balance: i64,
    pub spends: Vec<SaplingSpend>,
    pub outputs: Vec<SaplingOutput>,
    pub binding_signature: Signature,
}

/// Orchard action
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct OrchardAction {
    pub nullifier: Nullifier,
    pub cmu: Commitment,
    pub cv: ValueCommitment,
    pub cv_net: ValueCommitment,
    pub ephemeral_key: EphemeralKey,
    pub enc_ciphertext: Vec<u8>,
    pub out_ciphertext: Vec<u8>,
    pub proof: Vec<u8>,
    pub authorization: Signature,
}

/// Orchard bundle
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct OrchardBundle {
    pub actions: Vec<OrchardAction>,
    pub value_commitment: ValueCommitment,
    pub binding_signature: Signature,
}

/// Complete Zcash transaction
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct CompleteZcashTransaction {
    pub version: TransactionVersion,
    pub version_group_id: VersionGroupId,
    pub lock_time: LockTime,
    pub expiry_height: ExpiryHeight,
    pub transparent_inputs: Vec<TransparentInput>,
    pub transparent_outputs: Vec<TransparentOutput>,
    pub sapling_bundle: Option<SaplingBundle>,
    pub orchard_bundle: Option<OrchardBundle>,
    pub tx_hash: TxHash,
}

/// Validation result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct ValidationResult {
    pub is_valid: bool,
    pub total_input_value: Zatoshis,
    pub total_output_value: Zatoshis,
    pub fee: Zatoshis,
    pub transparent_balance: i64,
    pub sapling_balance: i64,
    pub orchard_balance: i64,
    pub nullifiers_valid: bool,
    pub commitments_valid: bool,
    pub signatures_valid: bool,
    pub zk_proofs_valid: bool,
    pub new_state_root: StateRoot,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: true,
            total_input_value: 0,
            total_output_value: 0,
            fee: 0,
            transparent_balance: 0,
            sapling_balance: 0,
            orchard_balance: 0,
            nullifiers_valid: true,
            commitments_valid: true,
            signatures_valid: true,
            zk_proofs_valid: true,
            new_state_root: [0u8; 32],
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Batch validation result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct BatchValidationResult {
    pub batch_valid: bool,
    pub total_transactions: usize,
    pub transaction_count: usize,
    pub valid_transactions: usize,
    pub invalid_transactions: usize,
    pub total_input_value: Zatoshis,
    pub total_output_value: Zatoshis,
    pub total_fee: Zatoshis,
    pub total_fees: Zatoshis,
    pub new_state_root: StateRoot,
    pub transaction_results: Vec<ValidationResult>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Block validation result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct BlockValidationResult {
    pub block_hash: TxHash,
    pub block_height: BlockHeight,
    pub transaction_count: usize,
    pub valid_transactions: usize,
    pub invalid_transactions: usize,
    pub block_valid: bool,
    pub total_input_value: Zatoshis,
    pub total_output_value: Zatoshis,
    pub total_fees: Zatoshis,
    pub new_state_root: StateRoot,
    pub transaction_results: Vec<ValidationResult>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Validation configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct ValidationConfig {
    pub check_signatures: bool,
    pub check_zk_proofs: bool,
    pub check_consensus_rules: bool,
    pub max_transaction_size: usize,
    pub max_batch_size: usize,
    pub max_inputs: usize,
    pub max_outputs: usize,
    pub max_fee: Zatoshis,
    pub min_fee: Zatoshis,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_signatures: true,
            check_zk_proofs: true,
            check_consensus_rules: true,
            max_transaction_size: 100_000, // 100KB
            max_batch_size: 1000,
            max_inputs: 1000,
            max_outputs: 1000,
            max_fee: 1_000_000, // 1 ZEC in zatoshis
            min_fee: 1000, // 0.00001 ZEC in zatoshis
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "host", derive(Serialize, Deserialize))]
pub struct PerformanceMetrics {
    pub validation_time_ms: u64,
    pub parsing_time_ms: u64,
    pub signature_verification_time_ms: u64,
    pub zk_proof_verification_time_ms: u64,
    pub state_update_time_ms: u64,
    pub total_time_ms: u64,
    pub memory_usage_bytes: usize,
}
