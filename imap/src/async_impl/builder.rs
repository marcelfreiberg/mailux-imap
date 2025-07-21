use rustls::RootCertStore;
use rustls::pki_types::ServerName;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::ImapError;

pub struct Builder {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Connector {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Client {
    stream: TlsStream<TcpStream>,
}

pub struct Session {
    _stream: TlsStream<TcpStream>,
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

    pub async fn connect(self) -> Result<Client, ImapError> {
        self.build().connect().await
    }
}

impl Connector {
    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub async fn connect(self) -> Result<Client, ImapError> {
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

                let connector = TlsConnector::from(Arc::new(config));
                let sock = TcpStream::connect(&self.addr).await
                    .map_err(|e| ImapError::Tls(e.to_string()))?;
                let mut stream = connector.connect(server_name, sock).await
                    .map_err(|e| ImapError::Tls(e.to_string()))?;
                
                // Since we have to read the greeting, we don't have to derive the TLS handshake
                // manually. The first read will derive the TLS handshake implicitly.
                Self::greeting(&mut stream).await?;

                tracing::info!("TLS connection established");
                
                Ok(Client { stream })
            }
            _ => Err(ImapError::Connection(
                "Connection type not implemented".to_string(),
            )),
        }
    }
    
    async fn greeting(stream: &mut TlsStream<TcpStream>) -> Result<(), ImapError> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("* OK") {
            return Err(ImapError::Connection(line));
        }

        Ok(())
    }
}

// pub fn connect_tls(addr: &str) -> Result<Client, ImapError> {
//     Builder::new(addr).tls().build().connect()
// }

// pub fn connect_starttls(addr: &str) -> Result<Client, ImapError> {
//     Builder::new(addr).starttls().build().connect()
// }

// pub fn connect_plain(addr: &str) -> Result<Client, ImapError> {
//     Builder::new(addr).plain().build().connect()
// }

impl Client {
    #[tracing::instrument(skip(self, pass))]
    pub async fn login(mut self, user: &str, pass: &str) -> Result<Session, ImapError> {
        tracing::info!("Attempting IMAP login");

        self.stream.write_all(format!("a001 LOGIN {} {}\r\n", user, pass).as_bytes()).await
            .map_err(|e| ImapError::Io(e.to_string()))?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        reader.read_line(&mut line).await
            .map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("* CAPABILITY") {
            return Err(ImapError::Connection(line));
        }
        
        line.clear();
        reader.read_line(&mut line).await
            .map_err(|e| ImapError::Io(e.to_string()))?;

        if !line.starts_with("a001 OK") {
            return Err(ImapError::Connection(line));
        }
        
        tracing::info!("IMAP login successful");

        Ok(Session { _stream: self.stream })
    }
}

impl Session {
    pub async fn fetch(&mut self, _mailbox: &str, _id: u32) -> Result<Messages, ImapError> {
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
