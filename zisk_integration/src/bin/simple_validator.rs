//! Simple Zcash Validator Binary
//! 
//! This binary demonstrates the basic functionality without ZisK dependencies.

use zisk_zcash_validator::simple_main::{SimpleValidator, SimpleTransaction, TxInput, TxOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Starting Simple Zcash Validator...");
    
    // Create validator
    let mut validator = SimpleValidator::new();
    
    // Add some initial UTXOs
    validator.utxos.insert(
        "initial_tx_1".to_string(),
        TxOutput {
            script_pubkey: "76a9141234567890abcdef1234567890abcdef12345678".to_string(),
            value: 1000000, // 0.01 ZEC
        }
    );
    
    // Create a test transaction
    let tx = SimpleTransaction {
        txid: "test_tx_001".to_string(),
        inputs: vec![TxInput {
            prev_txid: "initial_tx_1".to_string(),
            vout: 0,
            script_sig: "304402207f...".to_string(),
            value: 1000000,
        }],
        outputs: vec![
            TxOutput {
                script_pubkey: "76a914abcdef1234567890abcdef1234567890abcdef12".to_string(),
                value: 950000, // 0.0095 ZEC
            },
            TxOutput {
                script_pubkey: "76a9141234567890abcdef1234567890abcdef12345678".to_string(),
                value: 45000, // 0.00045 ZEC (change)
            },
        ],
        value: 950000,
        fee: 5000, // 0.00005 ZEC fee
    };
    
    // Validate the transaction
    let result = validator.validate_transaction(&tx);
    
    println!("📊 Validation Results:");
    println!("   Valid: {}", result.is_valid);
    println!("   Errors: {:?}", result.errors);
    println!("   State Root: {}", result.state_root);
    println!("   Timestamp: {}", result.timestamp);
    
    if result.is_valid {
        println!("✅ Transaction validation successful!");
        println!("🌳 New state root: {}", validator.get_state_root());
    } else {
        println!("❌ Transaction validation failed!");
    }
    
    println!("🎉 Simple validator completed successfully!");
    
    Ok(())
}