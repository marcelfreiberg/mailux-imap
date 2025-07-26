use rustls::StreamOwned;
use std::io::BufRead;
use std::io::Write;
use std::marker::PhantomData;
use std::net::TcpStream;

use imap::{ImapError, tls};
use imap::messages::{Message, Messages};

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
    stream: StreamOwned<rustls::ClientConnection, TcpStream>,
    state: PhantomData<State>,
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

    pub fn connect(self) -> Result<Client<Connected>, ImapError> {
        self.build().connect()
    }
}

impl Connector {
    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub fn connect(self) -> Result<Client<Connected>, ImapError> {
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

                Ok(Client { 
                    stream,
                    state: PhantomData,
                })
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

pub fn connect_tls(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).tls().build().connect()
}

pub fn connect_starttls(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).starttls().build().connect()
}

pub fn connect_plain(addr: &str) -> Result<Client<Connected>, ImapError> {
    Builder::new(addr).plain().build().connect()
}

impl Client<Connected> {
    #[tracing::instrument(skip(self, pass))]
    pub fn login(mut self, user: &str, pass: &str) -> Result<Client<Authenticated>, ImapError> {
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

        Ok(Client {
            stream: self.stream,
            state: PhantomData,
        })
    }
}

impl Client<Authenticated> {
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
