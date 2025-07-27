use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use memchr::memmem;
use std::collections::HashMap;
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use imap::messages::{Message, Messages};
use imap::parser::greeting;
use imap::tls;

pub struct Connected;
pub struct Authenticated;

const LINE_CAP: usize = 8 * 1024;
const GROW_STEP: usize = 2 * 1024; // 2 KiB increments (one TLS record fragment)

// Will be later replaced
struct Command {
    tag: String,
    text: String,
    reply: oneshot::Sender<Bytes>,
}

pub struct Connector {
    addr: String,
    conn_type: crate::ConnectionType,
}

pub struct Client<State> {
    cmd_tx: mpsc::Sender<Command>,
    unsol_rx: broadcast::Receiver<Bytes>,
    state: PhantomData<State>,
}

impl Connector {
    pub fn new(addr: &str, conn_type: crate::ConnectionType) -> Self {
        Self {
            addr: addr.to_owned(),
            conn_type,
        }
    }

    #[tracing::instrument(skip(self), fields(addr = %self.addr, conn_type = ?self.conn_type))]
    pub async fn connect(self) -> Result<Client<Connected>> {
        tracing::info!("Connecting to IMAP server");

        match self.conn_type {
            crate::ConnectionType::Tls => {
                let config = tls::create_tls_config();
                let server_name = tls::parse_server_name(&self.addr)?;

                let connector = TlsConnector::from(config);
                let sock = TcpStream::connect(&self.addr).await?;
                let stream = connector.connect(server_name, sock).await?;

                let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);
                let (unsol_tx, unsol_rx) = broadcast::channel::<Bytes>(64);
                let (greeting_tx, greeting_rx) = oneshot::channel::<Result<()>>();

                tokio::spawn(async move {
                    if let Err(e) = Self::run_imap_loop(stream, cmd_rx, unsol_tx, greeting_tx).await
                    {
                        tracing::error!("Error handling messages: {}", e);
                    }
                });

                greeting_rx.await??;

                Ok(Client::<Connected> {
                    cmd_tx,
                    unsol_rx,
                    state: PhantomData,
                })
            }
            _ => anyhow::bail!("Connection type not implemented"),
        }
    }

    async fn run_imap_loop(
        mut stream: TlsStream<TcpStream>,
        mut cmd_rx: mpsc::Receiver<Command>,
        unsol_tx: broadcast::Sender<Bytes>,
        greeting_tx: oneshot::Sender<Result<()>>,
    ) -> Result<()> {
        let mut buf = BytesMut::with_capacity(1024);

        // Handle greeting
        loop {
            // Check spare capacity before reading
            if buf.remaining_mut() == 0 {
                if buf.capacity() >= LINE_CAP {
                    anyhow::bail!("Greeting too long");
                }
                let add = GROW_STEP.min(LINE_CAP - buf.capacity());
                buf.reserve(add);
            }

            let n = stream.read_buf(&mut buf).await?;
            if n == 0 {
                anyhow::bail!("EOF before greeting");
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
                        anyhow::bail!("Greeting parse error: {}", e);
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
                n = stream.read_buf(&mut buf) => {
                    let n = n?;
                    if n == 0 { anyhow::bail!("Server closed connection") }

                    while let Some(pos) = memmem::find(&buf, b"\r\n") {
                        let line = buf.split_to(pos + 2).freeze();
                        Self::route_line(line, &unsol_tx, &mut pending);
                    }

                    if buf.remaining_mut() == 0 {
                        if buf.capacity() >= LINE_CAP {
                            anyhow::bail!("IMAP line exceeded {LINE_CAP} bytes");
                        }
                        let add = GROW_STEP.min(LINE_CAP - buf.capacity());
                        buf.reserve(add);
                    }
                }
                Some(cmd) = cmd_rx.recv() => {
                    stream.write_all(cmd.text.as_bytes()).await?;
                    stream.flush().await?;
                    pending.insert(cmd.tag, cmd.reply);
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
    ) {
        // Simple routing logic - this will be enhanced later with proper parsing
        let tag_end = line.iter().position(|&b| b == b' ').unwrap_or(line.len());
        let tag = &line[..tag_end];
        if let Some(tx) = pending.remove(unsafe { std::str::from_utf8_unchecked(tag) }) {
            let _ = tx.send(line);
            return;
        }
        
        let _ = unsol_tx.send(line);
    }
}

impl Client<Connected> {
    #[tracing::instrument(skip(self, _pass))]
    pub async fn login(self, _user: &str, _pass: &str) -> Result<Client<Authenticated>> {
        tracing::info!("Attempting IMAP login");

        Ok(Client::<Authenticated> {
            cmd_tx: self.cmd_tx,
            unsol_rx: self.unsol_rx,
            state: PhantomData,
        })
    }
}

impl Client<Authenticated> {
    pub async fn fetch(&mut self, _mailbox: &str, _id: u32) -> Result<Messages> {
        Ok(Messages::new(vec![
            Message::new("Subject1".to_string()),
            Message::new("Subject2".to_string()),
        ]))
    }
}
