use rustls::{RootCertStore, StreamOwned};
use rustls::pki_types::ServerName;
use std::net::TcpStream;
use std::sync::Arc;
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

pub struct Builder {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Connector {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Client {
    stream: StreamOwned<rustls::ClientConnection, TcpStream>,
}

pub struct Session {
    stream: StreamOwned<rustls::ClientConnection, TcpStream>,
}

pub struct Message {
    subject: String,
}

pub struct Messages {
    messages: Vec<Result<Message, ImapError>>,
}

enum ConnectionType {
    Tls,
    StartTls,
    Plain,
}

impl Builder {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
            conn_type: ConnectionType::Tls,
        }
    }

    pub fn tls(mut self) -> Self {
        self.conn_type = ConnectionType::Tls;
        self
    }

    pub fn starttls(mut self) -> Self {
        self.conn_type = ConnectionType::StartTls;
        self
    }

    pub fn plain(mut self) -> Self {
        self.conn_type = ConnectionType::Plain;
        self
    }

    pub fn build(self) -> Connector {
        Connector {
            addr: self.addr,
            conn_type: self.conn_type,
        }
    }

    pub fn connect(self) -> Result<Client, ImapError> {
        self.build().connect()
    }
}

impl Connector {
    pub fn connect(self) -> Result<Client, ImapError> {
        match self.conn_type {
            ConnectionType::Tls => {
                let root_store = RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                };

                let config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                let (host, _) = self
                    .addr
                    .rsplit_once(':')
                    .ok_or_else(|| ImapError::DnsName(self.addr.clone()))?;

                let server_name = ServerName::try_from(host.to_string())
                    .map_err(|e| ImapError::DnsName(e.to_string()))?;
                let conn = rustls::ClientConnection::new(Arc::new(config), server_name)
                    .map_err(|e| ImapError::Io(e.to_string()))?;
                let sock = TcpStream::connect(&self.addr)
                    .map_err(|e| ImapError::Tls(e.to_string()))?;
                let stream = rustls::StreamOwned::new(conn, sock);

                Ok(Client { stream })
            }
            _ => Err(ImapError::Connection(
                "Connection type not implemented".to_string(),
            )),
        }
    }
}

pub fn connect_tls(addr: &str) -> Result<Client, ImapError> {
    Builder::new(addr).tls().build().connect()
}

pub fn connect_starttls(addr: &str) -> Result<Client, ImapError> {
    Builder::new(addr).starttls().build().connect()
}

pub fn connect_plain(addr: &str) -> Result<Client, ImapError> {
    Builder::new(addr).plain().build().connect()
}

impl Client {
    pub fn login(self, user: &str, pass: &str) -> Result<Session, ImapError> {
        Ok(Session { stream: self.stream })
    }
}

impl Session {
    pub fn fetch(&mut self, mailbox: &str, id: u32) -> Result<Messages, ImapError> {
        Ok(Messages {
            messages: vec![
                Ok(Message {
                    subject: "Subject1".to_string(),
                }),
                Ok(Message {
                    subject: "Subject2".to_string(),
                }),
            ],
        })
    }
}

impl Message {
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl Messages {
    pub fn try_next(&mut self) -> Result<Option<Message>, ImapError> {
        if !self.messages.is_empty() {
            let result = self.messages.remove(0);
            match result {
                Ok(message) => Ok(Some(message)),
                Err(error) => Err(error),
            }
        } else {
            Ok(None)
        }
    }
}
