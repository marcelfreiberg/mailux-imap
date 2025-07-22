use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImapError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid address format: {0}")]
    InvalidAddressFormat(String),
    #[error("DNS name error: {0}")]
    InvalidDnsName(#[from] rustls::pki_types::InvalidDnsNameError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
