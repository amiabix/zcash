//! ZkVM Input Serializer
//! 
//! This module converts RPC responses from Zcash nodes into
//! the binary format expected by the ZisK zkVM.

use crate::core::*;
use crate::error::*;
use crate::bridge::node_bridge::{NodeBridge, NodeConfig, NodeUtxo};
use alloc::vec::Vec;
use alloc::string::String;
use serde_json::json;

/// ZkVM input serializer for converting RPC data to zkVM format
pub struct ZkvmInputSerializer {
    node_bridge: NodeBridge,
    current_height: u32,
}

/// Serialized batch input for zkVM
#[derive(Debug, Clone)]
pub struct SerializedBatchInput {
    /// Prior state root (32 bytes)
    pub prior_state_root: [u8; 32],
    /// Transaction batch hash (32 bytes)
    pub tx_batch_hash: [u8; 32],
    /// Block height (4 bytes)
    pub block_height: u32,
    /// Consensus branch ID (4 bytes)
    pub consensus_branch_id: u32,
    /// UTXOs with Merkle proofs
    pub utxos: Vec<UtxoEntry>,
    /// Raw transaction batch data
    pub tx_batch: Vec<u8>,
}

/// UTXO entry for zkVM input
#[derive(Debug, Clone)]
pub struct UtxoEntry {
    pub prev_txid: [u8; 32],
    pub prev_index: u32,
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub merkle_proof: Vec<[u8; 32]>,
    pub leaf_index: u64,
    pub height: u32,
    pub spendable: bool,
}

impl ZkvmInputSerializer {
    /// Create a new input serializer
    pub fn new(node_bridge: NodeBridge) -> Self {
        Self {
            node_bridge,
            current_height: 0,
        }
    }

    /// Serialize transaction batch for zkVM input
    pub async fn serialize_batch_input(
        &mut self,
        transactions: &[ZcashTransaction],
        prior_state_root: [u8; 32],
    ) -> ZcashResult<SerializedBatchInput> {
        // 1. Get current block height
        self.current_height = self.node_bridge.get_block_height().await?;
        
        // 2. Fetch all required UTXOs
        let mut utxos = Vec::new();
        for tx in transactions {
            for input in &tx.transparent_inputs {
                let utxo = self.node_bridge.fetch_utxo(input.prevout_hash, input.prevout_index).await?;
                let utxo_entry = self.convert_node_utxo_to_entry(utxo).await?;
                utxos.push(utxo_entry);
            }
        }
        
        // 3. Serialize transactions to binary
        let tx_batch = self.serialize_transactions_to_binary(transactions)?;
        
        // 4. Compute transaction batch hash
        let tx_batch_hash = self.compute_batch_hash(&tx_batch);
        
        Ok(SerializedBatchInput {
            prior_state_root,
            tx_batch_hash,
            block_height: self.current_height,
            consensus_branch_id: 0x892F2085, // Zcash consensus branch ID
            utxos,
            tx_batch,
        })
    }

    /// Convert NodeUtxo to UtxoEntry with Merkle proof
    async fn convert_node_utxo_to_entry(&self, utxo: NodeUtxo) -> ZcashResult<UtxoEntry> {
        // In a real implementation, this would fetch the Merkle proof
        // For now, create a mock proof
        let merkle_proof = self.generate_mock_merkle_proof(utxo.txid, utxo.vout).await?;
        
        Ok(UtxoEntry {
            prev_txid: utxo.txid,
            prev_index: utxo.vout,
            value: utxo.value,
            script_pubkey: utxo.script_pubkey,
            merkle_proof,
            leaf_index: 0, // Would be calculated from actual proof
            height: utxo.height,
            spendable: utxo.spendable,
        })
    }

    /// Generate mock Merkle proof (in production, fetch from node)
    async fn generate_mock_merkle_proof(&self, txid: [u8; 32], vout: u32) -> ZcashResult<Vec<[u8; 32]>> {
        // Mock implementation - in production would call gettxoutproof
        let mut proof = Vec::new();
        for i in 0..10 {
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            proof.push(hash);
        }
        Ok(proof)
    }

    /// Serialize transactions to binary format
    fn serialize_transactions_to_binary(&self, transactions: &[ZcashTransaction]) -> ZcashResult<Vec<u8>> {
        let mut data = Vec::new();
        
        // Transaction count (4 bytes LE)
        data.extend_from_slice(&(transactions.len() as u32).to_le_bytes());
        
        // Serialize each transaction
        for tx in transactions {
            self.serialize_transaction(&mut data, tx)?;
        }
        
        Ok(data)
    }

    /// Serialize single transaction to binary
    fn serialize_transaction(&self, data: &mut Vec<u8>, tx: &ZcashTransaction) -> ZcashResult<()> {
        // Version (4 bytes LE)
        data.extend_from_slice(&tx.version.to_le_bytes());
        
        // Version group ID (4 bytes LE)
        data.extend_from_slice(&tx.version_group_id.to_le_bytes());
        
        // Lock time (4 bytes LE)
        data.extend_from_slice(&tx.lock_time.to_le_bytes());
        
        // Expiry height (4 bytes LE)
        data.extend_from_slice(&tx.expiry_height.to_le_bytes());
        
        // Transparent input count (4 bytes LE)
        data.extend_from_slice(&(tx.transparent_inputs.len() as u32).to_le_bytes());
        
        // Serialize transparent inputs
        for input in &tx.transparent_inputs {
            self.serialize_transparent_input(data, input)?;
        }
        
        // Transparent output count (4 bytes LE)
        data.extend_from_slice(&(tx.transparent_outputs.len() as u32).to_le_bytes());
        
        // Serialize transparent outputs
        for output in &tx.transparent_outputs {
            self.serialize_transparent_output(data, output)?;
        }
        
        // Sapling bundle (if present)
        if let Some(ref sapling) = tx.sapling_bundle {
            self.serialize_sapling_bundle(data, sapling)?;
        }
        
        // Orchard bundle (if present)
        if let Some(ref orchard) = tx.orchard_bundle {
            self.serialize_orchard_bundle(data, orchard)?;
        }
        
        Ok(())
    }

    /// Serialize transparent input
    fn serialize_transparent_input(&self, data: &mut Vec<u8>, input: &TransparentInput) -> ZcashResult<()> {
        // Previous output hash (32 bytes)
        data.extend_from_slice(&input.prevout_hash);
        
        // Previous output index (4 bytes LE)
        data.extend_from_slice(&input.prevout_index.to_le_bytes());
        
        // Script signature length (4 bytes LE)
        data.extend_from_slice(&(input.script_sig.len() as u32).to_le_bytes());
        
        // Script signature
        data.extend_from_slice(&input.script_sig);
        
        // Sequence (4 bytes LE)
        data.extend_from_slice(&input.sequence.to_le_bytes());
        
        Ok(())
    }

    /// Serialize transparent output
    fn serialize_transparent_output(&self, data: &mut Vec<u8>, output: &TransparentOutput) -> ZcashResult<()> {
        // Value (8 bytes LE)
        data.extend_from_slice(&output.value.to_le_bytes());
        
        // Script public key length (4 bytes LE)
        data.extend_from_slice(&(output.script_pubkey.len() as u32).to_le_bytes());
        
        // Script public key
        data.extend_from_slice(&output.script_pubkey);
        
        Ok(())
    }

    /// Serialize Sapling bundle
    fn serialize_sapling_bundle(&self, data: &mut Vec<u8>, bundle: &SaplingBundle) -> ZcashResult<()> {
        // Value balance (8 bytes LE)
        data.extend_from_slice(&bundle.value_balance.to_le_bytes());
        
        // Spend count (4 bytes LE)
        data.extend_from_slice(&(bundle.spends.len() as u32).to_le_bytes());
        
        // Serialize spends
        for spend in &bundle.spends {
            self.serialize_sapling_spend(data, spend)?;
        }
        
        // Output count (4 bytes LE)
        data.extend_from_slice(&(bundle.outputs.len() as u32).to_le_bytes());
        
        // Serialize outputs
        for output in &bundle.outputs {
            self.serialize_sapling_output(data, output)?;
        }
        
        // Binding signature length (4 bytes LE)
        data.extend_from_slice(&(bundle.binding_signature.len() as u32).to_le_bytes());
        
        // Binding signature
        data.extend_from_slice(&bundle.binding_signature);
        
        Ok(())
    }

    /// Serialize Sapling spend
    fn serialize_sapling_spend(&self, data: &mut Vec<u8>, spend: &SaplingSpend) -> ZcashResult<()> {
        // Nullifier (32 bytes)
        data.extend_from_slice(&spend.nullifier);
        
        // Value commitment (32 bytes)
        data.extend_from_slice(&spend.cv);
        
        // Anchor (32 bytes)
        data.extend_from_slice(&spend.anchor);
        
        // Proof length (4 bytes LE)
        data.extend_from_slice(&(spend.proof.len() as u32).to_le_bytes());
        
        // Proof
        data.extend_from_slice(&spend.proof);
        
        // Spend description length (4 bytes LE)
        data.extend_from_slice(&(spend.spend_description.len() as u32).to_le_bytes());
        
        // Spend description
        data.extend_from_slice(&spend.spend_description);
        
        Ok(())
    }

    /// Serialize Sapling output
    fn serialize_sapling_output(&self, data: &mut Vec<u8>, output: &SaplingOutput) -> ZcashResult<()> {
        // Note commitment (32 bytes)
        data.extend_from_slice(&output.cmu);
        
        // Value commitment (32 bytes)
        data.extend_from_slice(&output.cv);
        
        // Ephemeral public key (32 bytes)
        data.extend_from_slice(&output.ephemeral_key);
        
        // Encrypted ciphertext length (4 bytes LE)
        data.extend_from_slice(&(output.enc_ciphertext.len() as u32).to_le_bytes());
        
        // Encrypted ciphertext
        data.extend_from_slice(&output.enc_ciphertext);
        
        // Out ciphertext length (4 bytes LE)
        data.extend_from_slice(&(output.out_ciphertext.len() as u32).to_le_bytes());
        
        // Out ciphertext
        data.extend_from_slice(&output.out_ciphertext);
        
        // Proof length (4 bytes LE)
        data.extend_from_slice(&(output.proof.len() as u32).to_le_bytes());
        
        // Proof
        data.extend_from_slice(&output.proof);
        
        Ok(())
    }

    /// Serialize Orchard bundle
    fn serialize_orchard_bundle(&self, data: &mut Vec<u8>, bundle: &OrchardBundle) -> ZcashResult<()> {
        // Action count (4 bytes LE)
        data.extend_from_slice(&(bundle.actions.len() as u32).to_le_bytes());
        
        // Serialize actions
        for action in &bundle.actions {
            self.serialize_orchard_action(data, action)?;
        }
        
        // Value commitment (32 bytes)
        data.extend_from_slice(&bundle.value_commitment);
        
        // Binding signature length (4 bytes LE)
        data.extend_from_slice(&(bundle.binding_signature.len() as u32).to_le_bytes());
        
        // Binding signature
        data.extend_from_slice(&bundle.binding_signature);
        
        Ok(())
    }

    /// Serialize Orchard action
    fn serialize_orchard_action(&self, data: &mut Vec<u8>, action: &OrchardAction) -> ZcashResult<()> {
        // Nullifier (32 bytes)
        data.extend_from_slice(&action.nullifier);
        
        // Value commitment (32 bytes)
        data.extend_from_slice(&action.cv);
        
        // Note commitment (32 bytes)
        data.extend_from_slice(&action.cmu);
        
        // Ephemeral key (32 bytes)
        data.extend_from_slice(&action.ephemeral_key);
        
        // Encrypted ciphertext length (4 bytes LE)
        data.extend_from_slice(&(action.enc_ciphertext.len() as u32).to_le_bytes());
        
        // Encrypted ciphertext
        data.extend_from_slice(&action.enc_ciphertext);
        
        // Out ciphertext length (4 bytes LE)
        data.extend_from_slice(&(action.out_ciphertext.len() as u32).to_le_bytes());
        
        // Out ciphertext
        data.extend_from_slice(&action.out_ciphertext);
        
        // Proof length (4 bytes LE)
        data.extend_from_slice(&(action.proof.len() as u32).to_le_bytes());
        
        // Proof
        data.extend_from_slice(&action.proof);
        
        Ok(())
    }

    /// Compute batch hash
    fn compute_batch_hash(&self, tx_batch: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(tx_batch);
        let first = hasher.finalize_reset();
        hasher.update(&first);
        let second = hasher.finalize_reset();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&second);
        hash
    }

    /// Convert to binary format for zkVM input
    pub fn to_binary(&self, batch_input: &SerializedBatchInput) -> ZcashResult<Vec<u8>> {
        let mut data = Vec::new();
        
        // Prior state root (32 bytes)
        data.extend_from_slice(&batch_input.prior_state_root);
        
        // Transaction batch hash (32 bytes)
        data.extend_from_slice(&batch_input.tx_batch_hash);
        
        // Block height (4 bytes LE)
        data.extend_from_slice(&batch_input.block_height.to_le_bytes());
        
        // Consensus branch ID (4 bytes LE)
        data.extend_from_slice(&batch_input.consensus_branch_id.to_le_bytes());
        
        // UTXO count (4 bytes LE)
        data.extend_from_slice(&(batch_input.utxos.len() as u32).to_le_bytes());
        
        // Serialize each UTXO
        for utxo in &batch_input.utxos {
            self.serialize_utxo_entry(&mut data, utxo)?;
        }
        
        // Transaction batch length (4 bytes LE)
        data.extend_from_slice(&(batch_input.tx_batch.len() as u32).to_le_bytes());
        
        // Transaction batch data
        data.extend_from_slice(&batch_input.tx_batch);
        
        Ok(data)
    }

    /// Serialize UTXO entry
    fn serialize_utxo_entry(&self, data: &mut Vec<u8>, utxo: &UtxoEntry) -> ZcashResult<()> {
        // Entry length (4 bytes LE) - will be filled after serialization
        let length_offset = data.len();
        data.extend_from_slice(&0u32.to_le_bytes());
        
        let start_offset = data.len();
        
        // Previous transaction ID (32 bytes)
        data.extend_from_slice(&utxo.prev_txid);
        
        // Previous output index (4 bytes LE)
        data.extend_from_slice(&utxo.prev_index.to_le_bytes());
        
        // Value (8 bytes LE)
        data.extend_from_slice(&utxo.value.to_le_bytes());
        
        // Script public key length (4 bytes LE)
        data.extend_from_slice(&(utxo.script_pubkey.len() as u32).to_le_bytes());
        
        // Script public key
        data.extend_from_slice(&utxo.script_pubkey);
        
        // Height (4 bytes LE)
        data.extend_from_slice(&utxo.height.to_le_bytes());
        
        // Spendable (1 byte)
        data.push(if utxo.spendable { 1 } else { 0 });
        
        // Merkle proof length (4 bytes LE)
        data.extend_from_slice(&(utxo.merkle_proof.len() as u32).to_le_bytes());
        
        // Merkle proof hashes
        for hash in &utxo.merkle_proof {
            data.extend_from_slice(hash);
        }
        
        // Leaf index (8 bytes LE)
        data.extend_from_slice(&utxo.leaf_index.to_le_bytes());
        
        // Update length field
        let entry_length = data.len() - start_offset;
        let length_bytes = (entry_length as u32).to_le_bytes();
        data[length_offset..length_offset + 4].copy_from_slice(&length_bytes);
        
        Ok(())
    }
}
