//! Error types for LogicCraft L1 core.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogicError {
    #[error("Gate or wire not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Combinational feedback loop detected without sequential DFF/Clock isolation: {0}")]
    CycleDetected(String),

    #[error("SQLite storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
