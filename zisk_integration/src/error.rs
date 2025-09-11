//! Error types and handling for ZisK-Zcash validation

use alloc::string::String;

/// Macro for creating parse errors
#[macro_export]
macro_rules! parse_error {
    ($err:expr) => {
        crate::error::ZcashValidationError::ParseError($err)
    };
}

/// Main error type for ZisK-Zcash validation operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZcashValidationError {
    /// Transaction parsing errors
    ParseError(ParseError),
    /// Validation rule violations
    ValidationError(ValidationError),
    /// Proof generation errors
    ProofError(ProofError),
    /// ZisK runtime errors
    ZiskError(ZiskError),
    /// IO and system errors
    SystemError(SystemError),
}

/// Transaction parsing specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid transaction format
    InvalidFormat,
    /// Unsupported transaction version
    UnsupportedVersion(u32),
    /// Invalid version group ID
    InvalidVersionGroup(u32),
    /// Malformed input/output data
    MalformedData,
    /// Insufficient data for parsing
    InsufficientData,
    /// Invalid script format
    InvalidScript,
    /// Invalid signature format
    InvalidSignature,
    /// Invalid commitment format
    InvalidCommitment,
    /// Invalid nullifier format
    InvalidNullifier,
}

/// Validation rule violation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Transaction version not supported
    UnsupportedVersion(u32),
    /// Lock time in the future
    FutureLockTime(u32),
    /// Invalid expiry height
    InvalidExpiryHeight(u32),
    /// Input value exceeds maximum
    InputValueTooLarge(u64),
    /// Output value exceeds maximum
    OutputValueTooLarge(u64),
    /// Fee too high
    FeeTooHigh(u64),
    /// Fee too low (dust)
    FeeTooLow(u64),
    /// Value conservation violation
    ValueConservationViolation {
        input_value: u64,
        output_value: u64,
        fee: u64,
    },
    /// Invalid script execution
    ScriptExecutionFailed,
    /// Invalid signature
    InvalidSignature,
    /// Double spend detected
    DoubleSpend,
    /// Invalid nullifier
    InvalidNullifier,
    /// Invalid commitment
    InvalidCommitment,
    /// Invalid anchor
    InvalidAnchor,
    /// Invalid ephemeral key
    InvalidEphemeralKey,
    /// Invalid value balance
    InvalidValueBalance(i64),
    /// Invalid shielded bundle
    InvalidShieldedBundle,
    /// Transaction too large
    TransactionTooLarge(usize),
    /// Too many inputs
    TooManyInputs(usize),
    /// Too many outputs
    TooManyOutputs(usize),
}

/// Proof generation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    /// ZisK compilation failed
    CompilationFailed(String),
    /// ROM setup failed
    RomSetupFailed(String),
    /// Proof generation timeout
    ProofTimeout,
    /// Insufficient memory for proof generation
    InsufficientMemory,
    /// Invalid witness data
    InvalidWitness,
    /// Proof verification failed
    VerificationFailed,
    /// Invalid proof format
    InvalidProofFormat,
}

/// ZisK runtime errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZiskError {
    /// RISC-V execution failed
    ExecutionFailed(String),
    /// Memory allocation failed
    MemoryAllocationFailed,
    /// Invalid input data
    InvalidInput,
    /// Invalid output data
    InvalidOutput,
    /// VM state corruption
    StateCorruption,
    /// Unsupported instruction
    UnsupportedInstruction,
}

/// System and IO errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemError {
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied,
    /// Disk full
    DiskFull,
    /// Network error
    NetworkError(String),
    /// Configuration error
    ConfigError(String),
    /// Resource exhausted
    ResourceExhausted,
    /// RPC error
    RpcError(String),
    /// Parse error
    ParseError(String),
}

impl core::fmt::Display for ZcashValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZcashValidationError::ParseError(e) => write!(f, "Parse error: {}", e),
            ZcashValidationError::ValidationError(e) => write!(f, "Validation error: {}", e),
            ZcashValidationError::ProofError(e) => write!(f, "Proof error: {}", e),
            ZcashValidationError::ZiskError(e) => write!(f, "ZisK error: {}", e),
            ZcashValidationError::SystemError(e) => write!(f, "System error: {}", e),
        }
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid transaction format"),
            ParseError::UnsupportedVersion(v) => write!(f, "Unsupported transaction version: {}", v),
            ParseError::InvalidVersionGroup(vg) => write!(f, "Invalid version group ID: 0x{:x}", vg),
            ParseError::MalformedData => write!(f, "Malformed transaction data"),
            ParseError::InsufficientData => write!(f, "Insufficient data for parsing"),
            ParseError::InvalidScript => write!(f, "Invalid script format"),
            ParseError::InvalidSignature => write!(f, "Invalid signature format"),
            ParseError::InvalidCommitment => write!(f, "Invalid commitment format"),
            ParseError::InvalidNullifier => write!(f, "Invalid nullifier format"),
        }
    }
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationError::UnsupportedVersion(v) => write!(f, "Unsupported version: {}", v),
            ValidationError::FutureLockTime(lt) => write!(f, "Future lock time: {}", lt),
            ValidationError::InvalidExpiryHeight(eh) => write!(f, "Invalid expiry height: {}", eh),
            ValidationError::InputValueTooLarge(v) => write!(f, "Input value too large: {} zatoshis", v),
            ValidationError::OutputValueTooLarge(v) => write!(f, "Output value too large: {} zatoshis", v),
            ValidationError::FeeTooHigh(fee) => write!(f, "Fee too high: {} zatoshis", fee),
            ValidationError::FeeTooLow(fee) => write!(f, "Fee too low: {} zatoshis", fee),
            ValidationError::ValueConservationViolation { input_value, output_value, fee } => {
                write!(f, "Value conservation violation: input={}, output={}, fee={}", 
                       input_value, output_value, fee)
            },
            ValidationError::ScriptExecutionFailed => write!(f, "Script execution failed"),
            ValidationError::InvalidSignature => write!(f, "Invalid signature"),
            ValidationError::DoubleSpend => write!(f, "Double spend detected"),
            ValidationError::InvalidNullifier => write!(f, "Invalid nullifier"),
            ValidationError::InvalidCommitment => write!(f, "Invalid commitment"),
            ValidationError::InvalidAnchor => write!(f, "Invalid anchor"),
            ValidationError::InvalidEphemeralKey => write!(f, "Invalid ephemeral key"),
            ValidationError::InvalidValueBalance(vb) => write!(f, "Invalid value balance: {}", vb),
            ValidationError::InvalidShieldedBundle => write!(f, "Invalid shielded bundle"),
            ValidationError::TransactionTooLarge(size) => write!(f, "Transaction too large: {} bytes", size),
            ValidationError::TooManyInputs(count) => write!(f, "Too many inputs: {}", count),
            ValidationError::TooManyOutputs(count) => write!(f, "Too many outputs: {}", count),
        }
    }
}

impl core::fmt::Display for ProofError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProofError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
            ProofError::RomSetupFailed(msg) => write!(f, "ROM setup failed: {}", msg),
            ProofError::ProofTimeout => write!(f, "Proof generation timeout"),
            ProofError::InsufficientMemory => write!(f, "Insufficient memory for proof generation"),
            ProofError::InvalidWitness => write!(f, "Invalid witness data"),
            ProofError::VerificationFailed => write!(f, "Proof verification failed"),
            ProofError::InvalidProofFormat => write!(f, "Invalid proof format"),
        }
    }
}

impl core::fmt::Display for ZiskError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZiskError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ZiskError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            ZiskError::InvalidInput => write!(f, "Invalid input data"),
            ZiskError::InvalidOutput => write!(f, "Invalid output data"),
            ZiskError::StateCorruption => write!(f, "VM state corruption"),
            ZiskError::UnsupportedInstruction => write!(f, "Unsupported instruction"),
        }
    }
}

impl core::fmt::Display for SystemError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SystemError::FileNotFound(path) => write!(f, "File not found: {}", path),
            SystemError::PermissionDenied => write!(f, "Permission denied"),
            SystemError::DiskFull => write!(f, "Disk full"),
            SystemError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SystemError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            SystemError::ResourceExhausted => write!(f, "Resource exhausted"),
            SystemError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            SystemError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

/// Result type for ZisK-Zcash operations
pub type ZcashResult<T> = Result<T, ZcashValidationError>;