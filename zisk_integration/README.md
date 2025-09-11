# ZisK-Zcash Transaction Validator

A ZisK zkVM implementation for Zcash transaction validation. This project provides the foundation for proving Zcash transaction validity in a zero-knowledge environment.

## Current Implementation Status

### What Works
- **Basic Transaction Parsing**: Parses Zcash transaction headers (version, version_group_id, lock_time, expiry_height)
- **ECDSA Signature Verification**: Uses k256 library for real ECDSA signature verification
- **Type System**: Core transaction data structures
- **State Management**: Sparse Merkle Tree implementation for UTXO tracking
- **Error Handling**: Basic error types and handling
- **No-std Compatibility**: Compiles for ZisK zkVM environment

### What's Missing (Critical)
- **Real Transaction Parsing**: Only parses headers, not inputs/outputs or Sapling/Orchard bundles
- **Real UTXO Validation**: SMT exists but not used in actual validation
- **Real Consensus Rules**: Only basic bounds checking
- **Real zk-SNARK Verification**: Sapling/Orchard proofs not verified
- **Real Proof Generation**: STARK proofs are deterministic hashes, not real proofs

## Project Structure

```
zisk_integration/
├── src/
│   ├── main.rs                 # Main zkVM program entry point
│   ├── core.rs                 # Core transaction types and structures
│   ├── error.rs                # Error types and handling
│   ├── secp_verify.rs          # ECDSA signature verification (real implementation)
│   ├── smt.rs                  # Sparse Merkle Tree for UTXO management
│   ├── utxo_validation.rs      # UTXO validation functions
│   ├── parsing/
│   │   ├── mod.rs              # Transaction parser (basic header parsing only)
│   │   ├── v4_parser.rs        # V4 parser (disabled due to compilation issues)
│   │   ├── v5_parser.rs        # V5 parser (placeholder)
│   │   ├── script_parser.rs    # Script parsing (placeholder)
│   │   └── crypto_utils.rs     # Cryptographic utilities
│   ├── validation/
│   │   ├── mod.rs              # Validation module (basic bounds checking)
│   │   ├── consensus_validator.rs  # Consensus rules (incomplete)
│   │   ├── transparent_validator.rs # Transparent validation (placeholder)
│   │   ├── sapling_validator.rs    # Sapling validation (placeholder)
│   │   ├── orchard_validator.rs    # Orchard validation (placeholder)
│   │   └── fee_validator.rs        # Fee validation (placeholder)
│   ├── proofs/
│   │   ├── mod.rs              # Proof module (placeholder)
│   │   ├── proof_generator.rs  # STARK proof generation (deterministic hashes)
│   │   └── proof_verifier.rs   # Proof verification (placeholder)
│   ├── verification/
│   │   ├── mod.rs              # zk-SNARK verification (basic structure validation)
│   │   └── sapling_verifier.rs # Sapling proof verification (placeholder)
│   ├── state/
│   │   ├── mod.rs              # State management (placeholder)
│   │   └── state_synchronizer.rs # State synchronization (placeholder)
│   ├── utils/
│   │   ├── mod.rs              # Utilities (placeholder)
│   │   ├── logging.rs          # Logging utilities (placeholder)
│   │   ├── performance.rs      # Performance monitoring (placeholder)
│   │   └── serialization.rs    # Serialization utilities (placeholder)
│   └── bin/
│       ├── block_validator.rs  # Block validation binary
│       ├── batch_validator.rs  # Batch validation binary
│       ├── single_validator.rs # Single transaction validator
│       ├── simple_validator.rs # Simple validator
│       ├── minimal_validator.rs # Minimal validator
│       └── static_validator.rs # Static validator
├── build.rs                    # Build script for ZisK target
├── Cargo.toml                  # Project dependencies and configuration
├── Dockerfile                  # Docker setup for testing
└── README.md                   # This file
```

## File Descriptions

### Core Files
- **main.rs**: The main zkVM program that runs inside ZisK. Contains the primary validation logic and calls other modules.
- **core.rs**: Defines the core data structures for Zcash transactions (CompleteZcashTransaction, TransparentInput, etc.)
- **error.rs**: Error types and handling for the validation system.

### Parsing Module
- **parsing/mod.rs**: Main transaction parser. Currently only parses transaction headers (version, version_group_id, lock_time, expiry_height). Does NOT parse inputs/outputs or Sapling/Orchard bundles.
- **parsing/v4_parser.rs**: V4 transaction parser (disabled due to compilation issues with no_std environment).
- **parsing/v5_parser.rs**: V5 transaction parser (placeholder, not implemented).
- **parsing/script_parser.rs**: Bitcoin script parsing (placeholder, not implemented).
- **parsing/crypto_utils.rs**: Cryptographic utilities (placeholder, not implemented).

### Validation Module
- **validation/mod.rs**: Main validation module with basic bounds checking.
- **validation/consensus_validator.rs**: Zcash consensus rules (incomplete implementation).
- **validation/transparent_validator.rs**: Transparent transaction validation (placeholder).
- **validation/sapling_validator.rs**: Sapling transaction validation (placeholder).
- **validation/orchard_validator.rs**: Orchard transaction validation (placeholder).
- **validation/fee_validator.rs**: Fee validation (placeholder).

### Cryptographic Components
- **secp_verify.rs**: Real ECDSA signature verification using k256 library. This is one of the few working components.
- **smt.rs**: Sparse Merkle Tree implementation for UTXO management. Code exists but not integrated into validation.
- **utxo_validation.rs**: UTXO validation functions with Merkle proof verification.

### Proof System
- **proofs/proof_generator.rs**: STARK proof generation. Currently generates deterministic hashes, not real STARK proofs.
- **proofs/proof_verifier.rs**: Proof verification (placeholder).
- **verification/mod.rs**: zk-SNARK verification for Sapling/Orchard (basic structure validation only).
- **verification/sapling_verifier.rs**: Sapling proof verification (placeholder).

### Binary Targets
- **bin/block_validator.rs**: Block-level transaction validation.
- **bin/batch_validator.rs**: Batch transaction processing.
- **bin/single_validator.rs**: Single transaction validation.
- **bin/simple_validator.rs**: Simple validation demo.
- **bin/minimal_validator.rs**: Minimal validation example.
- **bin/static_validator.rs**: Static validation example.

## Current Limitations

1. **Incomplete Transaction Parsing**: Only parses transaction headers, not the actual transaction content.
2. **No Real UTXO Validation**: SMT implementation exists but is not used in validation.
3. **No Real Consensus Rules**: Only basic bounds checking, missing Zcash-specific rules.
4. **No Real zk-SNARK Verification**: Sapling and Orchard proofs are not actually verified.
5. **No Real Proof Generation**: STARK proofs are deterministic hashes, not cryptographic proofs.
6. **No Integration Testing**: Not tested with real Zcash transactions.

## Build Instructions

```bash
# Clone the repository
git clone https://github.com/amiabix/zcash.git
cd zcash/zisk_integration

# Build the library
cargo build --lib

# Build for ZisK target (if ZisK toolchain is installed)
cargo build --target riscv64ima-zisk-zkvm-elf
```

## Development Status

This is a **development implementation** with significant limitations. The code compiles and runs, but does not provide complete Zcash transaction validation. Most components are placeholders or incomplete implementations.

## License

This project is licensed under the MIT License.

## Acknowledgments

- ZisK team for the zkVM infrastructure
- Zcash Foundation for protocol specifications
- Rust community for excellent tooling