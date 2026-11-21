//! Error types for the boxes component.

/// Errors in image build planning.
#[derive(Debug, thiserror::Error)]
pub enum BoxesError {
    /// Invalid image configuration.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
