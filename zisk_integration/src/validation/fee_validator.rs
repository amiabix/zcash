//! Fee validation for Zcash transactions

use crate::core::*;
use crate::error::*;
use alloc::vec::Vec;
use alloc::string::String;

/// Fee validator
pub struct FeeValidator {
    config: ValidationConfig,
}

impl FeeValidator {
    /// Create a new fee validator
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate transaction fees
    pub fn validate(&self, transaction: &ZcashTransaction, result: &mut ValidationResult) -> ZcashResult<()> {
        // Calculate fee
        let total_input = result.total_input_value as i64;
        let total_output = result.total_output_value as i64;
        let sapling_balance = result.sapling_balance;
        let orchard_balance = result.orchard_balance;

        let fee = total_input + sapling_balance + orchard_balance - total_output;
        
        if fee < 0 {
            result.is_valid = false;
            result.errors.push(format!("Negative fee: {}", fee));
            return Err(validation_error!(ValidationError::FeeTooLow(fee as u64)));
        }

        result.fee = fee as u64;

        // Validate fee range
        if result.fee > self.config.max_fee {
            result.is_valid = false;
            result.errors.push(format!("Fee too high: {} zatoshis", result.fee));
            return Err(validation_error!(ValidationError::FeeTooHigh(result.fee)));
        }

        if result.fee < self.config.min_fee {
            result.warnings.push(format!("Fee very low: {} zatoshis", result.fee));
        }

        // Check for dust outputs
        for (i, output) in transaction.transparent_outputs.iter().enumerate() {
            if output.value < 1000 { // Dust threshold
                result.warnings.push(format!("Output {}: Dust output detected: {} zatoshis", i, output.value));
            }
        }

        Ok(())
    }
}
