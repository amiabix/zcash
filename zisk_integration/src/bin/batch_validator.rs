//! Batch Zcash transaction validator for ZisK

#![no_main]
#![no_std]

extern crate alloc;

use zisk_zcash_validator::*;
use zisk_zcash_validator::parsing::ZcashTransaction;
use zisk_zcash_validator::proofs::StarkProof;
use ziskos::{read_input, set_output};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Main entry point for batch transaction validation
#[no_mangle]
fn main() {
    // Read input data from ZisK
    let input_data = read_input();
    
    // Create validator
    let validator = ZcashValidator::new(ValidationConfig::default());
    
    // Parse batch
    let parser = parsing::TransactionParser::new();
    let transactions = match parser.parse_batch(&input_data) {
        Ok(txs) => txs,
        Err(e) => {
            // Output error and exit
            set_output(0, 0); // Error flag
            set_output(1, 0); // Error code
            return;
        }
    };
    
    // Validate batch
    let batch_result = match validator.validate_batch(&transactions) {
        Ok(result) => result,
        Err(e) => {
            // Output error and exit
            // Temporary implementation - in production this would call ziskos::set_output()
            let _ = format!("Validation error: {}", e);
            return;
        }
    };
    
    // Generate batch proof
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
    output_batch_result(&batch_result);
    output_proof_info(&proof);
}

/// Output batch validation result
fn output_batch_result(result: &BatchValidationResult) {
    set_output(0, result.batch_valid as u32);
    set_output(1, result.total_transactions as u32);
    set_output(2, result.valid_transactions as u32);
    set_output(3, result.invalid_transactions as u32);
    set_output(4, (result.total_input_value >> 32) as u32);
    set_output(5, result.total_input_value as u32);
    set_output(6, (result.total_output_value >> 32) as u32);
    set_output(7, result.total_output_value as u32);
    set_output(8, (result.total_fee >> 32) as u32);
    set_output(9, result.total_fee as u32);
    set_output(10, result.warnings.len() as u32);
    set_output(11, result.errors.len() as u32);
    
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
