use thiserror::Error;

#[derive(Error, Debug)]
pub enum XmppError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("File transfer error: {0}")]
    FileTransferError(String),
    
    #[error("Invalid JID: {0}")]
    InvalidJid(String),
    
    #[error("Network timeout")]
    TimeoutError,
    
    #[error("TLS error: {0}")]
    TlsError(String),
}

pub type Result<T> = std::result::Result<T, XmppError>;