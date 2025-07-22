use bytes::{Buf, BytesMut};
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead};

use crate::ImapError;
use crate::messages::{Message, Messages};
use crate::tls;

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
    state: PhantomData<State>,
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
                let config = tls::create_tls_config();
                let server_name = tls::parse_server_name(&self.addr)?;

                let connector = TlsConnector::from(config);
                let sock = TcpStream::connect(&self.addr).await?;
                let stream = connector.connect(server_name, sock).await?;

                let mut framed = FramedRead::new(stream, ImapCodec::new());

                // Since we have to read the greeting, we don't have to derive the TLS handshake
                // manually. The first read will derive the TLS handshake implicitly.
                Self::handle_greeting(&mut framed).await?;

                tracing::info!("TLS connection established");

                Ok(Client {
                    framed,
                    state: PhantomData,
                })
            }
            _ => Err(ImapError::ConnectionFailed(
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
            .ok_or_else(|| ImapError::ConnectionFailed("EOF while reading greeting".to_string()))??;

        match resp {
            crate::parser::Response::Greeting(greeting) => match greeting.status {
                crate::parser::Status::Ok => {
                    tracing::info!("Received OK greeting from server");
                    Ok(())
                }
                _ => Err(ImapError::ConnectionFailed(
                    "Invalid greeting from server".to_string(),
                )),
            },
            _ => Err(ImapError::ConnectionFailed(
                "Expected greeting from server".to_string(),
            )),
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
    pub async fn login(mut self, user: &str, pass: &str) -> Result<Client<Authenticated>, ImapError> {
        tracing::info!("Attempting IMAP login");

        self.framed
            .get_mut()
            .write_all(format!("a001 LOGIN {} {}\r\n", user, pass).as_bytes())
            .await?;

        while let Some(result) = self.framed.next().await {
            match result? {
                crate::parser::Response::Tagged { tag, status, .. } if tag.as_ref() == b"a001" => {
                    match status {
                        crate::parser::Status::Ok => {
                            tracing::info!("IMAP login successful");
                            return Ok(Client {
                                framed: self.framed,
                                state: PhantomData,
                            });
                        }
                        _ => {
                            return Err(ImapError::ConnectionFailed("Login failed".to_string()));
                        }
                    }
                }
                crate::parser::Response::Greeting(greeting) => {
                    if matches!(greeting.status, crate::parser::Status::Bye) {
                        return Err(ImapError::ConnectionFailed(
                            "Server closed connection".to_string(),
                        ));
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        Err(ImapError::ConnectionFailed(
            "Connection closed unexpectedly".to_string(),
        ))
    }
}

impl Client<Authenticated> {
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
