use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImapError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("DNS name error: {0}")]
    DnsName(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("IO error: {0}")]
    Io(String),
}