use anyhow::{Context, Result};
use bytes::{BufMut, Bytes, BytesMut};
use memchr::memmem;
use std::collections::HashMap;
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::{next_tag, ConnectedState, AuthenticatedState};

use imap::commands::CommandBuilder;
use imap::messages::{Message, Messages};
use imap::parser::greeting;
use imap::tls;

const LINE_CAP: usize = 8 * 1024;
const GROW_STEP: usize = 2 * 1024; // 2 KiB increments (one TLS record fragment)

pub struct Connector {
    addr: String,
    conn_type: crate::ConnectionType,
}

pub struct Client<State> {
    cmd_tx: mpsc::Sender<String>,
    unsol_rx: broadcast::Receiver<Bytes>,
    _state: PhantomData<State>,
}

impl Connector {
    pub fn new(addr: &str, conn_type: crate::ConnectionType) -> Self {
        Self {
            addr: addr.to_owned(),
            conn_type,
        }
    }

    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub async fn connect(self) -> Result<Client<ConnectedState>> {
        tracing::info!("Connecting to IMAP server");

        match self.conn_type {
            crate::ConnectionType::Tls => {
                let config = tls::create_tls_config();
                let server_name = tls::parse_server_name(&self.addr).with_context(|| {
                    format!("Failed to parse server name from address: {}", self.addr)
                })?;

                let connector = TlsConnector::from(config);
                let sock = TcpStream::connect(&self.addr).await.with_context(|| {
                    format!("Failed to establish TCP connection to {}", self.addr)
                })?;
                let stream = connector
                    .connect(server_name, sock)
                    .await
                    .with_context(|| {
                        format!("Failed to establish TLS connection to {}", self.addr)
                    })?;

                let (cmd_tx, cmd_rx) = mpsc::channel::<String>(32);
                let (unsol_tx, unsol_rx) = broadcast::channel::<Bytes>(64);
                let (greeting_tx, greeting_rx) = oneshot::channel::<Result<()>>();

                tokio::spawn(async move {
                    if let Err(e) = Self::run_imap_loop(stream, cmd_rx, unsol_tx, greeting_tx).await
                    {
                        tracing::error!("Error handling messages: {}", e);
                    }
                });

                greeting_rx
                    .await
                    .context("Greeting handler task panicked or was cancelled")?
                    .context("Failed to process IMAP greeting")?;

                Ok(Client::<ConnectedState> {
                    cmd_tx,
                    unsol_rx,
                    _state: PhantomData,
                })
            }
            _ => anyhow::bail!("Connection type {:?} not implemented", self.conn_type),
        }
    }

    async fn run_imap_loop(
        mut stream: TlsStream<TcpStream>,
        mut cmd_rx: mpsc::Receiver<String>,
        unsol_tx: broadcast::Sender<Bytes>,
        greeting_tx: oneshot::Sender<Result<()>>,
    ) -> Result<()> {
        let mut buf = BytesMut::with_capacity(1024);

        // Handle greeting
        loop {
            // Check spare capacity before reading
            if buf.remaining_mut() == 0 {
                if buf.capacity() >= LINE_CAP {
                    anyhow::bail!(
                        "IMAP greeting exceeded maximum line length of {} bytes",
                        LINE_CAP
                    );
                }
                let add = GROW_STEP.min(LINE_CAP - buf.capacity());
                buf.reserve(add);
            }

            let n = stream
                .read_buf(&mut buf)
                .await
                .context("Failed to read data while waiting for IMAP greeting")?;
            if n == 0 {
                anyhow::bail!("Server closed connection before sending greeting");
            }

            if let Some(pos) = memmem::find(&buf, b"\r\n") {
                let line = buf.split_to(pos + 2).freeze();
                match greeting::try_parse(&line) {
                    Ok(Some(_greeting)) => {
                        let _ = greeting_tx.send(Ok(()));
                        break;
                    }
                    Ok(None) | Err(imap::parser::ParserError::Incomplete) => continue,
                    Err(e) => {
                        let err = e.to_string();
                        let _ = greeting_tx.send(Err(e.into()));
                        anyhow::bail!("Failed to parse IMAP greeting: {}", err);
                    }
                }
            }
        }

        // Ensure we have spare capacity before entering main loop
        if buf.remaining_mut() == 0 {
            let add = GROW_STEP.min(LINE_CAP - buf.capacity());
            buf.reserve(add);
        }

        let mut pending: HashMap<String, oneshot::Sender<Bytes>> = HashMap::new();

        // Main IMAP loop
        loop {
            tokio::select! {
                result = stream.read_buf(&mut buf) => {
                    let n = result.context("Failed to read data from IMAP server")?;
                    if n == 0 {
                        anyhow::bail!("IMAP server closed connection unexpectedly")
                    }

                    while let Some(pos) = memmem::find(&buf, b"\r\n") {
                        let line = buf.split_to(pos + 2).freeze();
                        Self::route_line(line, &unsol_tx, &mut pending)?;
                    }

                    if buf.remaining_mut() == 0 {
                        if buf.capacity() >= LINE_CAP {
                            anyhow::bail!("IMAP response line exceeded maximum length of {} bytes", LINE_CAP);
                        }
                        let add = GROW_STEP.min(LINE_CAP - buf.capacity());
                        buf.reserve(add);
                    }
                }
                Some(cmd) = cmd_rx.recv() => {
                    stream.write_all(cmd.as_bytes()).await
                        .with_context(|| format!("Failed to send IMAP command: {}", cmd))?;
                    stream.flush().await
                        .with_context(|| format!("Failed to flush IMAP command: {}", cmd))?;
                    // TODO: Handle command responses - pending functionality needs to be redesigned
                    // pending.insert(cmd.tag().to_string(), ???);
                }
                else => break,
            }
        }
        Ok(())
    }

    fn route_line(
        line: Bytes,
        unsol_tx: &broadcast::Sender<Bytes>,
        pending: &mut HashMap<String, oneshot::Sender<Bytes>>,
    ) -> Result<()> {
        // Simple routing logic - this will be enhanced later with proper parsing
        let tag_end = line.iter().position(|&b| b == b' ').unwrap_or(line.len());
        let tag = &line[..tag_end];

        // Convert tag bytes to string safely
        let tag_str =
            std::str::from_utf8(tag).context("IMAP response tag contains invalid UTF-8")?;

        if let Some(tx) = pending.remove(tag_str) {
            let _ = tx.send(line);
            return Ok(());
        }

        let _ = unsol_tx.send(line);
        Ok(())
    }
}

impl Client<ConnectedState> {
    #[tracing::instrument(skip(self, pass))]
    pub async fn login(mut self, user: &str, pass: &str) -> Result<Client<AuthenticatedState>> {
        tracing::info!("Attempting IMAP login");

        let login_command = CommandBuilder::new(&next_tag())
            .login()
            .username(user)
            .password(pass);

        self.cmd_tx
            .send(login_command.as_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send login command: {}", e))?;

        let response = self
            .unsol_rx
            .recv()
            .await
            .context("Login command timed out")?;
        tracing::debug!("Login response: {:?}", String::from_utf8_lossy(&response));
        
        // TODO: parse response

        Ok(Client::<AuthenticatedState> {
            cmd_tx: self.cmd_tx,
            unsol_rx: self.unsol_rx,
            _state: PhantomData,
        })
    }
}

impl Client<AuthenticatedState> {
    pub async fn fetch(&mut self, _mailbox: &str, _id: u32) -> Result<Messages> {
        Ok(Messages::new(vec![
            Message::new("Subject1".to_string()),
            Message::new("Subject2".to_string()),
        ]))
    }
}
