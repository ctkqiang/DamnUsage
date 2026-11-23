use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("provider not found: {0}")]
    ProviderNotFound(String),
    #[error("unsupported provider kind: {0}")]
    UnsupportedProvider(String),
    #[error("missing required credential field: {0}")]
    MissingCredential(String),
    #[error("provider request failed ({provider}): {message}")]
    ProviderRequestFailed { provider: String, message: String },
    #[error("provider response parse failed ({provider}): {message}")]
    ProviderParseFailed { provider: String, message: String },
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("secret vault error: {0}")]
    VaultError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
