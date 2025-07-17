use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImapError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Authentication failed")]
    Authentication,
    #[error("Invalid mailbox")]
    InvalidMailbox,
    #[error("Network error: {0}")]
    Network(String),
}

pub struct Builder {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Client {
    addr: String,
    conn_type: ConnectionType,
}

pub struct Session;

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

    pub fn build(self) -> Result<Client, ImapError> {
        Ok(Client {
            addr: self.addr,
            conn_type: self.conn_type,
        })
    }
}

impl Client {
    pub fn login(self, user: &str, pass: &str) -> Result<Session, ImapError> {
        Ok(Session)
    }
}

impl Session {
    pub fn fetch(&mut self, mailbox: &str, id: u32) -> Result<Messages, ImapError> {
        Ok(Messages{
            messages: vec![
                Ok(Message{ subject: "Subject1".to_string()}), 
                Ok(Message{ subject: "Subject2".to_string()})
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
