use rustls::StreamOwned;
use std::io::BufRead;
use std::io::Write;
use std::net::TcpStream;

use crate::ImapError;
use crate::messages::{Message, Messages};
use crate::tls;

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
    _stream: StreamOwned<rustls::ClientConnection, TcpStream>,
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
                let config = tls::create_tls_config();
                let server_name = tls::parse_server_name(&self.addr)?;

                let conn = rustls::ClientConnection::new(config, server_name)?;
                let sock = TcpStream::connect(&self.addr)?;
                let mut stream = rustls::StreamOwned::new(conn, sock);

                // Since we have to read the greeting, we don't have to derive the TLS handshake
                // manually. The first read will derive the TLS handshake implicitly.
                Self::handle_greeting(&mut stream)?;

                tracing::info!("TLS connection established");

                Ok(Client { stream })
            }
            _ => Err(ImapError::ConnectionFailed(
                "Connection type not implemented".to_string(),
            )),
        }
    }

    fn handle_greeting(
        stream: &mut StreamOwned<rustls::ClientConnection, TcpStream>,
    ) -> Result<(), ImapError> {
        let mut line = String::new();
        stream.read_line(&mut line)?;

        if !line.starts_with("* OK") {
            return Err(ImapError::ConnectionFailed(line));
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

        self.stream
            .write_all(format!("a001 LOGIN {} {}\r\n", user, pass).as_bytes()) ?;

        let mut line = String::new();
        self.stream
            .read_line(&mut line)?;

        if !line.starts_with("* CAPABILITY") {
            return Err(ImapError::Connection(line));
        }

        line.clear();
        self.stream
            .read_line(&mut line)?;

        if !line.starts_with("a001 OK") {
            return Err(ImapError::ConnectionFailed(line));
        }

        tracing::info!("IMAP login successful");

        Ok(Session {
            _stream: self.stream,
        })
    }
}

impl Session {
    pub fn fetch(&mut self, _mailbox: &str, _id: u32) -> Result<Messages, ImapError> {
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
