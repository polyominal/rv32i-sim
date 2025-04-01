use std::path::PathBuf;

use thiserror::Error;

/// Top-level error type for the simulator
#[derive(Error, Debug)]
pub enum SimulatorError {
    #[error("Failed to load ELF file: {0}")]
    ElfLoadError(#[from] ElfError),

    #[error("CPU execution error: {0}")]
    ExecutionError(#[from] ExecutionError),

    #[error("Memory error: {0}")]
    MemoryError(#[from] MemoryError),

    #[error("Invalid instruction: {0:032b} at PC={1:#010x}")]
    InvalidInstructionError(u32, u32),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

/// Errors related to ELF file operations
#[derive(Error, Debug)]
pub enum ElfError {
    #[error("Failed to read ELF file '{0}': {1}")]
    FileReadError(PathBuf, #[source] std::io::Error),

    #[error("Failed to parse ELF file '{0}': {1}")]
    ParseError(PathBuf, String),

    #[error("Invalid ELF format: {0}")]
    InvalidFormat(String),

    #[error("Memory address out of bounds: {0:#010x}")]
    AddressOutOfBounds(u32),

    #[error("Invalid ELF machine type: {0}")]
    InvalidMachine(u16),
}

/// Errors related to CPU execution
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Stack overflow: SP={0:#010x}, base={1:#010x}, size={2:#x}")]
    StackOverflow(u32, u32, u32),

    #[error("Unknown system call: {0}")]
    UnknownSystemCall(i32),

    #[error("Execution limit reached: {0} instructions")]
    ExecutionLimitReached(u64),

    #[error("Branch prediction failure: predicted={0}, actual={1}")]
    BranchPredictionFailure(bool, bool),
}

/// Errors related to memory operations
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory access error at address {address:#010x}: {kind}")]
    AccessError { address: u32, kind: MemoryErrorKind },

    #[error("Invalid memory alignment: address {0:#010x} is not aligned to {1} bytes")]
    AlignmentError(u32, u32),

    #[error("Page not allocated: {0:#010x}")]
    PageNotAllocated(u32),

    #[error("Cache inconsistency detected at level {0}: {1}")]
    CacheInconsistency(usize, String),
}

/// Specific kinds of memory errors
#[derive(Error, Debug)]
pub enum MemoryErrorKind {
    #[error("Attempted to read from unallocated memory")]
    ReadUnallocated,

    #[error("Attempted to write to unallocated memory")]
    WriteUnallocated,

    #[error("Attempted to access memory outside addressable range")]
    OutOfBounds,

    #[error("Invalid access size: {0}")]
    InvalidSize(u32),
}

/// Trait for converting standard Result into SimulatorResult
pub trait IntoSimulatorResult<T> {
    fn into_simulator_result(self) -> Result<T, SimulatorError>;
}

/// Type alias for Result with SimulatorError
pub type SimulatorResult<T> = Result<T, SimulatorError>;
