# ZisK-Zcash Integration

This directory contains the integration of ZisK zkVM with Zcash for programmable privacy circuits.

## Architecture Overview

The ZisK integration provides:

1. **Programmable Privacy Circuits**: RISC-V programs that implement shielded transaction logic
2. **STARK Proof System**: Replaces Groth16/Halo2 with ZisK's STARK proofs
3. **Distributed Proving**: Leverages ZisK's distributed proving infrastructure
4. **Backwards Compatibility**: Works alongside existing Sapling/Orchard pools

## Components

### Circuit Programs
- `circuits/spend_circuit/`: ZisK RISC-V program for shielded spends
- `circuits/output_circuit/`: ZisK RISC-V program for shielded outputs

### Transaction Structure
- `primitives/zisk_transaction.h`: ZisK transaction components (spends, outputs, bundles)
- Extended `primitives/transaction.h` with ZisK bundle support

### Proof System
- `proof_verifier.h/cpp`: ZisK STARK proof verification
- `consensus_validation.h/cpp`: Consensus validation for ZisK transactions

### Transaction Building
- `transaction_builder.h/cpp`: ZisK transaction construction and proof generation

## Usage Example

```cpp
#include "zisk/transaction_builder.h"

// Create ZisK transaction builder
ZiskTransactionBuilder builder(chainParams, nHeight, keystore, coinsView, cs_coinsView);

// Add ZisK spend
ZiskSpendInfo spendInfo(noteValue, noteRandomness, spendingKey, authPath, commitmentRoot);
builder.AddZiskSpend(spendInfo);

// Add ZisK output
builder.AddZiskOutput(recipientAddress, value, memo);

// Build transaction
CTransaction tx = builder.Build();
```

## Circuit Development

ZisK circuits are written in Rust and compiled to RISC-V:

```rust
#[zisk_main]
fn shielded_spend(
    note_value: u64,
    note_randomness: [u8; 32], 
    auth_path: Vec<[u8; 32]>,
    spending_key: [u8; 32],
    nullifier: [u8; 32],
    commitment_root: [u8; 32],
    value_commitment: [u8; 32],
) -> ZiskResult {
    // Circuit logic here
    Ok(())
}
```

## Integration Status

This is a proof-of-concept implementation demonstrating the architecture for integrating ZisK with Zcash. The actual implementation would require:

1. Complete ZisK SDK implementation
2. Cryptographic primitive implementations (Poseidon, Pedersen, etc.)
3. Integration with ZisK's distributed proving infrastructure
4. Comprehensive testing and security auditing

## Future Work

- [ ] Complete ZisK SDK implementation
- [ ] Implement cryptographic primitives
- [ ] Add distributed proving support
- [ ] Create migration tools for existing pools
- [ ] Performance optimization
- [ ] Security audit
