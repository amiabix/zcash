//! Block Zcash transaction validator for ZisK

#![no_main]
#![no_std]

extern crate alloc;

use zisk_zcash_validator::*;
use ziskos::{read_input, set_output};
use alloc::vec::Vec;
use alloc::string::String;

/// Main entry point for block transaction validation
#[no_mangle]
fn main() {
    // Read input data
    let input_data = read_input();
    
    // Create validator
    let validator = ZcashValidator::new(ValidationConfig::default());
    
    // Parse block data (simplified - in real implementation would parse block header)
    let block_height = parse_block_height(&input_data);
    let block_hash = parse_block_hash(&input_data);
    let transactions = parse_block_transactions(&input_data);
    
    // Validate all transactions in block
    let mut block_result = BlockValidationResult {
        block_height,
        block_hash,
        transaction_count: transactions.len(),
        valid_transactions: 0,
        invalid_transactions: 0,
        block_valid: true,
        total_input_value: 0,
        total_output_value: 0,
        total_fees: 0,
        transaction_results: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    
    // Validate each transaction
    for (i, transaction) in transactions.iter().enumerate() {
        let tx_result = match validator.validate_single(transaction) {
            Ok(result) => result,
            Err(e) => {
                block_result.block_valid = false;
                block_result.invalid_transactions += 1;
                block_result.errors.push(format!("Transaction {} validation failed: {}", i, e));
                continue;
            }
        };
        
        if tx_result.is_valid {
            block_result.valid_transactions += 1;
        } else {
            block_result.invalid_transactions += 1;
            block_result.block_valid = false;
        }
        
        block_result.total_input_value += tx_result.total_input_value;
        block_result.total_output_value += tx_result.total_output_value;
        block_result.total_fees += tx_result.fee;
        block_result.transaction_results.push(tx_result);
    }
    
    // Generate block proof
    let proof = match validator.generate_batch_proof(&transactions) {
        Ok(p) => p,
        Err(e) => {
            // Output error and exit
            set_output(0, 0); // Error flag
            set_output(1, 2); // Error code
            return;
        }
    };
    
    // Output results
    output_block_result(&block_result);
    output_proof_info(&proof);
}

/// Parse block height from input data
fn parse_block_height(data: &[u8]) -> BlockHeight {
    if data.len() < 4 {
        return 0;
    }
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Parse block hash from input data
fn parse_block_hash(data: &[u8]) -> TxHash {
    let mut hash = [0u8; 32];
    if data.len() >= 36 {
        hash.copy_from_slice(&data[4..36]);
    }
    hash
}

/// Parse block transactions from input data
fn parse_block_transactions(data: &[u8]) -> Vec<ZcashTransaction> {
    let parser = parsing::TransactionParser::new();
    
    // Skip block header (36 bytes)
    if data.len() <= 36 {
        return Vec::new();
    }
    
    let tx_data = &data[36..];
    
    // Try to parse as batch
    match parser.parse_batch(tx_data) {
        Ok(transactions) => transactions,
        Err(_) => {
            // If batch parsing fails, try single transaction
            match parser.parse_transaction(tx_data) {
                Ok(tx) => vec![tx],
                Err(_) => Vec::new(),
            }
        }
    }
}

/// Output block validation result
fn output_block_result(result: &BlockValidationResult) {
    set_output(0, result.block_valid as u32);
    set_output(1, result.block_height);
    set_output(2, result.transaction_count as u32);
    set_output(3, result.valid_transactions as u32);
    set_output(4, result.invalid_transactions as u32);
    set_output(5, (result.total_input_value >> 32) as u32);
    set_output(6, result.total_input_value as u32);
    set_output(7, (result.total_output_value >> 32) as u32);
    set_output(8, result.total_output_value as u32);
    set_output(9, (result.total_fees >> 32) as u32);
    set_output(10, result.total_fees as u32);
    set_output(11, result.warnings.len() as u32);
    set_output(12, result.errors.len() as u32);
    
    // Output individual transaction results
    for (i, tx_result) in result.transaction_results.iter().enumerate() {
        let base_offset = 100 + (i * 20);
        set_output(base_offset, tx_result.is_valid as u32);
        set_output(base_offset + 1, (tx_result.total_input_value >> 32) as u32);
        set_output(base_offset + 2, tx_result.total_input_value as u32);
        set_output(base_offset + 3, (tx_result.total_output_value >> 32) as u32);
        set_output(base_offset + 4, tx_result.total_output_value as u32);
        set_output(base_offset + 5, (tx_result.fee >> 32) as u32);
        set_output(base_offset + 6, tx_result.fee as u32);
        set_output(base_offset + 7, tx_result.transparent_balance as u32);
        set_output(base_offset + 8, tx_result.sapling_balance as u32);
        set_output(base_offset + 9, tx_result.orchard_balance as u32);
        set_output(base_offset + 10, tx_result.nullifiers_valid as u32);
        set_output(base_offset + 11, tx_result.commitments_valid as u32);
        set_output(base_offset + 12, tx_result.warnings.len() as u32);
        set_output(base_offset + 13, tx_result.errors.len() as u32);
    }
}

/// Output proof information
fn output_proof_info(proof: &StarkProof) {
    set_output(20, proof.metadata.riscv_cycles as u32);
    set_output(21, (proof.metadata.riscv_cycles >> 32) as u32);
    set_output(22, proof.metadata.memory_usage as u32);
    set_output(23, (proof.metadata.memory_usage >> 32) as u32);
    set_output(24, proof.metadata.proof_size as u32);
    set_output(25, (proof.metadata.proof_size >> 32) as u32);
    set_output(26, proof.metadata.compressed_size as u32);
    set_output(27, (proof.metadata.compressed_size >> 32) as u32);
    set_output(28, (proof.metadata.generation_time_us >> 32) as u32);
    set_output(29, proof.metadata.generation_time_us as u32);
    set_output(30, (proof.metadata.verification_time_us >> 32) as u32);
    set_output(31, proof.metadata.verification_time_us as u32);
}
