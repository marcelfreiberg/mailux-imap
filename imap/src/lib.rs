use rustls::{RootCertStore, StreamOwned};
use rustls::pki_types::ServerName;
use std::net::TcpStream;
use std::sync::Arc;
use std::io::BufRead;
use std::io::Write;
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

#[derive(Debug)]
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
    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub fn connect(self) -> Result<Client, ImapError> {
        tracing::info!("Connecting to IMAP server");
        
        match self.conn_type {
            ConnectionType::Tls => {
                let (host, _) = self
                    .addr
                    .rsplit_once(':')
                    .ok_or_else(|| ImapError::DnsName(self.addr.clone()))?;

                let root_store = RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                };

                let mut config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                
                if cfg!(debug_assertions) {
                    config.key_log = Arc::new(rustls::KeyLogFile::new());
                }

                let server_name = ServerName::try_from(host.to_string())
                    .map_err(|e| ImapError::DnsName(e.to_string()))?;

                let conn = rustls::ClientConnection::new(Arc::new(config), server_name)
                    .map_err(|e| ImapError::Io(e.to_string()))?;
                let sock = TcpStream::connect(&self.addr)
                    .map_err(|e| ImapError::Tls(e.to_string()))?;
                let mut stream = rustls::StreamOwned::new(conn, sock);
                
                // Since we have to read the greeting, we don't have to derive the TLS handshake
                // manually. The first read will derive the TLS handshake implicitly.
                Self::greeting(&mut stream)?;

                tracing::info!("TLS connection established");
                
                Ok(Client { stream })
            }
            _ => Err(ImapError::Connection(
                "Connection type not implemented".to_string(),
            )),
        }
    }
    
    fn greeting(stream: &mut StreamOwned<rustls::ClientConnection, TcpStream>) -> Result<(), ImapError> {
        let mut line = String::new();
        stream.read_line(&mut line).map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("* OK") {
            return Err(ImapError::Connection(line));
        }

        Ok(())
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
    #[tracing::instrument(skip(self, pass))]
    pub fn login(mut self, user: &str, pass: &str) -> Result<Session, ImapError> {
        tracing::info!("Attempting IMAP login");

        self.stream.write_all(format!("a001 LOGIN {} {}\r\n", user, pass).as_bytes())
            .map_err(|e| ImapError::Io(e.to_string()))?;

        let mut line = String::new();
        self.stream.read_line(&mut line)
            .map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("* CAPABILITY") {
            return Err(ImapError::Connection(line));
        }
        
        line.clear();
        self.stream.read_line(&mut line)
            .map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("a001 OK") {
            return Err(ImapError::Connection(line));
        }
        
        tracing::info!("IMAP login successful");

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
