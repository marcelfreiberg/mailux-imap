use bytes::{Buf, BytesMut};
use rustls::RootCertStore;
use rustls::pki_types::ServerName;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead};

use crate::ImapError;
use crate::messages::{Message, Messages};

// Connection states
pub struct Connected;
pub struct Authenticated;

pub struct Builder {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Connector {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Client<State = Connected> {
    framed: FramedRead<TlsStream<TcpStream>, ImapCodec>,
    _state: PhantomData<State>,
}

pub struct Session {
    framed: FramedRead<TlsStream<TcpStream>, ImapCodec>,
}

#[derive(Debug)]
enum ConnectionType {
    Tls,
    StartTls,
    Plain,
}

struct ImapCodec;

impl ImapCodec {
    fn new() -> Self {
        Self
    }
}

impl Decoder for ImapCodec {
    type Item = crate::parser::OwnedResponse;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        let parse_result = crate::parser::try_parse_response(buf);
        
        match parse_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))? {
            Some((resp, cnt)) => {
                buf.advance(cnt);
                Ok(Some(resp))
            }
            None => Ok(None),
        }
    }
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

    pub async fn connect(self) -> Result<Client<Connected>, ImapError> {
        self.build().connect().await
    }
}

impl Connector {
    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub async fn connect(self) -> Result<Client<Connected>, ImapError> {
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
                let sock = TcpStream::connect(&self.addr)
                    .await
                    .map_err(|e| ImapError::Tls(e.to_string()))?;
                let stream = connector
                    .connect(server_name, sock)
                    .await
                    .map_err(|e| ImapError::Tls(e.to_string()))?;

                let mut framed = FramedRead::new(stream, ImapCodec::new());

                // Since we have to read the greeting, we don't have to derive the TLS handshake
                // manually. The first read will derive the TLS handshake implicitly.
                Self::handle_greeting(&mut framed).await?;

                tracing::info!("TLS connection established");

                Ok(Client {
                    framed,
                    _state: PhantomData,
                })
            }
            _ => Err(ImapError::Connection(
                "Connection type not implemented".to_string(),
            )),
        }
    }

    async fn handle_greeting(
        framed: &mut FramedRead<TlsStream<TcpStream>, ImapCodec>,
    ) -> Result<(), ImapError> {
        let resp = framed
            .next()
            .await
            .ok_or_else(|| ImapError::Connection("EOF while reading greeting".to_string()))?
            .map_err(|e| ImapError::Io(e.to_string()))?;

        match resp {
            crate::parser::Response::Greeting(greeting) => {
                match greeting.status {
                    crate::parser::Status::Ok => {
                        tracing::info!("Received OK greeting from server");
                        Ok(())
                    }
                    _ => Err(ImapError::Connection("Invalid greeting from server".to_string())),
                }
            }
            _ => Err(ImapError::Connection("Expected greeting from server".to_string())),
        }
    }
}

pub async fn connect_tls(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).tls().build().connect().await
}

pub async fn connect_starttls(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).starttls().build().connect().await
}

pub async fn connect_plain(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).plain().build().connect().await
}

impl Client<Connected> {
    #[tracing::instrument(skip(self, pass))]
    pub async fn login(mut self, user: &str, pass: &str) -> Result<Session, ImapError> {
        tracing::info!("Attempting IMAP login");

        self.framed
            .get_mut()
            .write_all(format!("a001 LOGIN {} {}\r\n", user, pass).as_bytes())
            .await
            .map_err(|e| ImapError::Io(e.to_string()))?;

        while let Some(result) = self.framed.next().await {
            let resp = result.map_err(|e| ImapError::Io(e.to_string()))?;

            match resp {
                crate::parser::Response::Tagged { tag, status, .. } if tag.as_ref() == b"a001" => {
                    match status {
                        crate::parser::Status::Ok => {
                            tracing::info!("IMAP login successful");
                            return Ok(Session {
                                framed: self.framed,
                            });
                        }
                        _ => {
                            return Err(ImapError::Connection("Login failed".to_string()));
                        }
                    }
                }
                crate::parser::Response::Greeting(greeting) => {
                    if matches!(greeting.status, crate::parser::Status::Bye) {
                        return Err(ImapError::Connection("Server closed connection".to_string()));
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        Err(ImapError::Connection("Connection closed unexpectedly".to_string()))
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
