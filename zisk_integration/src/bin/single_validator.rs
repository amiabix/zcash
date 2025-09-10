//! Single Zcash transaction validator for ZisK

#![no_main]
#![no_std]

extern crate alloc;

use zisk_zcash_validator::*;
use ziskos::{read_input, set_output};
use alloc::vec::Vec;
use alloc::string::String;

/// Main entry point for single transaction validation
#[no_mangle]
fn main() {
    // Read input data
    let input_data = read_input();
    
    // Create validator
    let validator = ZcashValidator::new(ValidationConfig::default());
    
    // Parse transaction (simplified)
    let transaction = parse_transaction(&input_data);
    
    // Validate transaction
    let validation_result = match validator.validate_single(&transaction) {
        Ok(result) => result,
        Err(_) => {
            // Output error and exit
            set_output(0, 0); // Error flag
            set_output(1, 1); // Error code
            return;
        }
    };
    
    // Generate proof
    let proof = match validator.generate_proof(&transaction) {
        Ok(p) => p,
        Err(_) => {
            // Output error and exit
            set_output(0, 0); // Error flag
            set_output(1, 2); // Error code
            return;
        }
    };
    
    // Output results
    output_validation_result(&validation_result);
    output_proof_info(&proof);
}

/// Parse transaction from input data (simplified)
fn parse_transaction(data: &[u8]) -> ZcashTransaction {
    if data.len() < 16 {
        return create_default_transaction();
    }

    let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let version_group_id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let lock_time = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let expiry_height = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

    // Parse inputs and outputs (simplified)
    let input_count = if data.len() > 16 { data[16] as usize } else { 0 };
    let output_count = if data.len() > 17 { data[17] as usize } else { 0 };

    let mut transparent_inputs = Vec::new();
    let mut transparent_outputs = Vec::new();

    // Parse inputs (simplified)
    for i in 0..input_count {
        if data.len() > 18 + i * 40 {
            let mut prevout_hash = [0u8; 32];
            prevout_hash.copy_from_slice(&data[18 + i * 40..18 + i * 40 + 32]);
            
            transparent_inputs.push(TransparentInput {
                prevout_hash,
                prevout_index: 0,
                script_sig: Vec::new(),
                sequence: 0xFFFFFFFF,
            });
        }
    }

    // Parse outputs (simplified)
    for i in 0..output_count {
        if data.len() > 18 + input_count * 40 + i * 40 {
            let value = u64::from_le_bytes([
                data[18 + input_count * 40 + i * 40],
                data[18 + input_count * 40 + i * 40 + 1],
                data[18 + input_count * 40 + i * 40 + 2],
                data[18 + input_count * 40 + i * 40 + 3],
                data[18 + input_count * 40 + i * 40 + 4],
                data[18 + input_count * 40 + i * 40 + 5],
                data[18 + input_count * 40 + i * 40 + 6],
                data[18 + input_count * 40 + i * 40 + 7],
            ]);
            
            transparent_outputs.push(TransparentOutput {
                value,
                script_pubkey: Vec::new(),
            });
        }
    }

    ZcashTransaction {
        version,
        version_group_id,
        lock_time,
        expiry_height,
        transparent_inputs,
        transparent_outputs,
        sapling_bundle: None,
        orchard_bundle: None,
    }
}

/// Create default transaction
fn create_default_transaction() -> ZcashTransaction {
    ZcashTransaction {
        version: 4,
        version_group_id: 0x892F2085,
        lock_time: 0,
        expiry_height: 0,
        transparent_inputs: Vec::new(),
        transparent_outputs: Vec::new(),
        sapling_bundle: None,
        orchard_bundle: None,
    }
}

/// Output validation result
fn output_validation_result(result: &ValidationResult) {
    set_output(0, result.is_valid as u32);
    set_output(1, (result.total_input_value >> 32) as u32);
    set_output(2, result.total_input_value as u32);
    set_output(3, (result.total_output_value >> 32) as u32);
    set_output(4, result.total_output_value as u32);
    set_output(5, (result.fee >> 32) as u32);
    set_output(6, result.fee as u32);
    set_output(7, result.transparent_balance as u32);
    set_output(8, result.sapling_balance as u32);
    set_output(9, result.orchard_balance as u32);
    set_output(10, result.nullifiers_valid as u32);
    set_output(11, result.commitments_valid as u32);
    set_output(12, result.warnings.len() as u32);
    set_output(13, result.errors.len() as u32);
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