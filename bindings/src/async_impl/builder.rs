use anyhow::Result;
use crate::async_impl::{Connector, Client, Connected};

pub struct Builder {
    addr: String,
    conn_type: crate::ConnectionType,
}

impl Builder {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
            conn_type: crate::ConnectionType::Tls,
        }
    }

    pub fn tls(mut self) -> Self {
        self.conn_type = crate::ConnectionType::Tls;
        self
    }

    pub fn starttls(mut self) -> Self {
        self.conn_type = crate::ConnectionType::StartTls;
        self
    }

    pub fn plain(mut self) -> Self {
        self.conn_type = crate::ConnectionType::Plain;
        self
    }

    pub fn build(self) -> Connector {
        Connector::new(&self.addr, self.conn_type)
    }

    pub async fn connect(
        self,
    ) -> Result<Client<Connected>> {
        self.build().connect().await
    }
}

pub async fn connect_tls(addr: &str) -> Result<Client<Connected>> {
    Builder::new(addr).tls().build().connect().await
}

pub async fn connect_starttls(addr: &str) -> Result<Client<Connected>> {
    Builder::new(addr).starttls().build().connect().await
}

pub async fn connect_plain(addr: &str) -> Result<Client<Connected>> {
    Builder::new(addr).plain().build().connect().await
}
