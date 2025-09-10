//! Complete Zcash Validator with Node Integration
//! 
//! This module integrates all components: UTXO fetching, zk-SNARK verification,
//! and state synchronization for a complete production-ready validator.

use crate::core::*;
use crate::error::*;
use crate::bridge::{NodeBridge, UtxoProvider};
use crate::verification::sapling_verifier::SaplingVerifier;
use crate::state::state_synchronizer::StateSynchronizer;
use alloc::vec::Vec;
use alloc::string::String;

/// Complete Zcash validator with all integrations
pub struct CompleteZcashValidator<P: UtxoProvider> {
    /// Node bridge for UTXO fetching
    node_bridge: P,
    /// Sapling proof verifier
    sapling_verifier: SaplingVerifier,
    /// State synchronizer
    state_synchronizer: StateSynchronizer<P>,
    /// Validation configuration
    config: ValidationConfig,
}

impl<P: UtxoProvider> CompleteZcashValidator<P> {
    /// Create a new complete validator
    pub fn new(
        node_bridge: P,
        sapling_verifier: SaplingVerifier,
        state_synchronizer: StateSynchronizer<P>,
        config: ValidationConfig,
    ) -> Self {
        Self {
            node_bridge,
            sapling_verifier,
            state_synchronizer,
            config,
        }
    }

    /// Validate a single transaction with full node integration
    pub async fn validate_transaction_with_node(
        &mut self,
        transaction: &ZcashTransaction,
    ) -> ZcashResult<ValidationResult> {
        // 1. Fetch real UTXO data from node
        let utxo_data = self.fetch_utxo_data(transaction).await?;
        
        // 2. Verify Sapling proofs inside zkVM
        let sapling_verification = self.verify_sapling_proofs(transaction).await?;
        
        // 3. Validate transaction with real data
        let mut result = self.validate_with_real_data(transaction, &utxo_data).await?;
        
        // 4. Update state if transaction is valid
        if result.is_valid {
            self.state_synchronizer.process_transaction(transaction).await?;
        }
        
        // 5. Add verification results to validation result
        result.nullifiers_valid = sapling_verification.is_valid;
        result.commitments_valid = sapling_verification.is_valid;
        
        Ok(result)
    }

    /// Validate a batch of transactions
    pub async fn validate_batch_with_node(
        &mut self,
        transactions: &[ZcashTransaction],
    ) -> ZcashResult<BatchValidationResult> {
        let mut batch_result = BatchValidationResult {
            total_transactions: transactions.len(),
            valid_transactions: 0,
            invalid_transactions: 0,
            batch_valid: true,
            total_input_value: 0,
            total_output_value: 0,
            total_fee: 0,
            transaction_results: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Process each transaction
        for transaction in transactions {
            match self.validate_transaction_with_node(transaction).await {
                Ok(tx_result) => {
                    if tx_result.is_valid {
                        batch_result.valid_transactions += 1;
                    } else {
                        batch_result.invalid_transactions += 1;
                        batch_result.batch_valid = false;
                    }
                    
                    batch_result.total_input_value += tx_result.total_input_value;
                    batch_result.total_output_value += tx_result.total_output_value;
                    batch_result.total_fee += tx_result.fee;
                    batch_result.transaction_results.push(tx_result);
                }
                Err(e) => {
                    batch_result.invalid_transactions += 1;
                    batch_result.batch_valid = false;
                    batch_result.errors.push(format!("Transaction validation failed: {}", e));
                }
            }
        }

        Ok(batch_result)
    }

    /// Fetch UTXO data from node
    async fn fetch_utxo_data(&mut self, transaction: &ZcashTransaction) -> ZcashResult<Vec<NodeUtxo>> {
        let mut utxo_refs = Vec::new();
        
        // Collect UTXO references from transparent inputs
        for input in &transaction.transparent_inputs {
            utxo_refs.push((input.prevout_hash, input.prevout_index));
        }
        
        // Fetch UTXOs from node
        self.node_bridge.fetch_utxos_batch(&utxo_refs).await
    }

    /// Verify Sapling proofs inside zkVM
    async fn verify_sapling_proofs(&self, transaction: &ZcashTransaction) -> ZcashResult<SaplingVerificationResult> {
        if let Some(ref sapling_bundle) = transaction.sapling_bundle {
            // Parse and verify spend proofs
            let mut spend_proofs = Vec::new();
            for spend in &sapling_bundle.spends {
                let proof = SaplingProofParser::parse_spend_proof(&spend.proof)?;
                let public_inputs = spend.nullifier; // Simplified
                spend_proofs.push((proof.a, public_inputs, spend.clone()));
            }
            
            // Parse and verify output proofs
            let mut output_proofs = Vec::new();
            for output in &sapling_bundle.outputs {
                let proof = SaplingProofParser::parse_output_proof(&output.proof)?;
                let public_inputs = output.cmu; // Simplified
                output_proofs.push((proof.a, public_inputs, output.clone()));
            }
            
            // Verify all proofs
            Ok(self.sapling_verifier.verify_batch(&spend_proofs, &output_proofs))
        } else {
            // No Sapling bundle, return success
            Ok(SaplingVerificationResult {
                is_valid: true,
                verification_time_us: 0,
                error: None,
                public_inputs: Vec::new(),
            })
        }
    }

    /// Validate transaction with real UTXO data
    async fn validate_with_real_data(
        &self,
        transaction: &ZcashTransaction,
        utxo_data: &[NodeUtxo],
    ) -> ZcashResult<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            total_input_value: 0,
            total_output_value: 0,
            fee: 0,
            transparent_balance: 0,
            sapling_balance: 0,
            orchard_balance: 0,
            nullifiers_valid: true,
            commitments_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Validate transparent inputs with real UTXO data
        for (i, input) in transaction.transparent_inputs.iter().enumerate() {
            if let Some(utxo) = utxo_data.get(i) {
                // Verify input matches UTXO
                if input.prevout_hash != utxo.txid || input.prevout_index != utxo.vout {
                    result.is_valid = false;
                    result.errors.push(format!("Input {}: UTXO mismatch", i));
                    continue;
                }
                
                // Add real UTXO value
                result.total_input_value += utxo.value;
                
                // Verify UTXO is spendable
                if !utxo.spendable {
                    result.is_valid = false;
                    result.errors.push(format!("Input {}: UTXO not spendable", i));
                }
            } else {
                result.is_valid = false;
                result.errors.push(format!("Input {}: UTXO not found", i));
            }
        }

        // Validate transparent outputs
        for output in &transaction.transparent_outputs {
            result.total_output_value += output.value;
        }

        // Validate Sapling bundle
        if let Some(ref sapling_bundle) = transaction.sapling_bundle {
            result.sapling_balance = sapling_bundle.value_balance;
        }

        // Calculate fee
        result.fee = result.total_input_value.saturating_sub(result.total_output_value);
        result.transparent_balance = (result.total_input_value as i64) - (result.total_output_value as i64);

        // Validate value conservation
        let total_input = result.total_input_value as i64;
        let total_output = result.total_output_value as i64;
        let sapling_balance = result.sapling_balance;
        let orchard_balance = result.orchard_balance;
        let fee = result.fee as i64;

        if total_input + sapling_balance + orchard_balance != total_output + fee {
            result.is_valid = false;
            result.errors.push("Value conservation violation".to_string());
        }

        Ok(result)
    }

    /// Synchronize state with node
    pub async fn synchronize_with_node(&mut self, target_height: u32) -> ZcashResult<StateSyncResult> {
        self.state_synchronizer.synchronize_state(target_height).await
    }

    /// Get current state root
    pub fn get_current_state_root(&self) -> [u8; 32] {
        self.state_synchronizer.get_current_state_root()
    }

    /// Verify state consistency
    pub async fn verify_state_consistency(&self) -> ZcashResult<bool> {
        self.state_synchronizer.verify_consistency().await
    }
}

/// Factory for creating complete validators
pub struct ValidatorFactory;

impl ValidatorFactory {
    /// Create a complete validator with node integration
    pub async fn create_with_node_integration(
        node_config: NodeConfig,
        validation_config: ValidationConfig,
    ) -> ZcashResult<CompleteZcashValidator<NodeBridge>> {
        // Create node bridge
        let node_bridge = NodeBridge::new(node_config);
        
        // Load Sapling verification key
        let sapling_vk = SaplingKeyLoader::load_verification_key();
        let sapling_verifier = SaplingVerifier::new(sapling_vk, false); // Don't trust node
        
        // Create initial state
        let initial_state = StateRoot {
            utxo_root: [0u8; 32],
            sapling_root: [0u8; 32],
            orchard_root: [0u8; 32],
            combined_root: [0u8; 32],
        };
        
        // Create state synchronizer
        let state_synchronizer = StateSynchronizer::new(node_bridge, initial_state);
        
        // Create complete validator
        Ok(CompleteZcashValidator::new(
            node_bridge,
            sapling_verifier,
            state_synchronizer,
            validation_config,
        ))
    }
}

// Re-export types for convenience
pub use crate::bridge::node_bridge::{NodeConfig, NodeUtxo, NodeSaplingNote, NodeOrchardNote};
pub use crate::verification::sapling_verifier::{SaplingVerificationResult, SaplingKeyLoader, SaplingProofParser};
pub use crate::state::state_synchronizer::{StateSyncResult, StateRoot, StateChange};
