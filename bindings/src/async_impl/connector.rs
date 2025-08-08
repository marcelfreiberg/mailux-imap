use anyhow::{Context, Result};
use bytes::{BufMut, Bytes, BytesMut};
use memchr::memmem;
use std::collections::VecDeque;
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::{AuthenticatedState, ConnectedState, next_tag};

use imap::commands::CommandBuilder;
use imap::parser::{fetch, greeting};
use imap::tls;
use imap::types::command::{SequenceBound, SequenceSet};
use imap::types::response::{Envelope, FetchData};

const LINE_CAP: usize = 8 * 1024;
const GROW_STEP: usize = 2 * 1024; // 2 KiB increments (one TLS record fragment)

pub struct Connector {
    addr: String,
    conn_type: crate::ConnectionType,
}

pub struct Client<State> {
    cmd_tx: mpsc::Sender<CommandMessage>,
    unsol_rx: broadcast::Receiver<Bytes>,
    _state: PhantomData<State>,
}

struct CommandMessage {
    tag: String,
    command: String,
    responder: oneshot::Sender<Vec<Bytes>>, // all lines collected for this command (untagged + completion)
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

                let (cmd_tx, cmd_rx) = mpsc::channel::<CommandMessage>(32);
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
        mut cmd_rx: mpsc::Receiver<CommandMessage>,
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

        #[derive(Debug)]
        struct ActiveCommand {
            tag: String,
            responder: oneshot::Sender<Vec<Bytes>>,
            collected: Vec<Bytes>,
        }

        let mut active: Option<ActiveCommand> = None;
        let mut queue: VecDeque<CommandMessage> = VecDeque::new();

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

                        // Broadcast raw line
                        let _ = unsol_tx.send(line.clone());

                        // If active command and this is a continuation request, we may need to write literal content.
                        // For now, we just collect lines and detect completion; literal handling is covered by
                        // parsing ENVELOPE which can include literals on the server side, but they arrive inline.

                        if let Some(active_cmd) = &mut active {
                            active_cmd.collected.push(line.clone());
                            if is_tagged_completion(&line, &active_cmd.tag) {
                                let collected = std::mem::take(&mut active_cmd.collected);
                                let responder = std::mem::replace(&mut active_cmd.responder, oneshot::channel().0);
                                let _ = responder.send(collected);
                                active = None;

                                if let Some(next) = queue.pop_front() {
                                    stream.write_all(next.command.as_bytes()).await
                                        .with_context(|| format!("Failed to send IMAP command: {}", next.command))?;
                                    stream.flush().await
                                        .with_context(|| format!("Failed to flush IMAP command: {}", next.command))?;
                                    active = Some(ActiveCommand { tag: next.tag, responder: next.responder, collected: Vec::new() });
                                }
                            }
                        }
                    }

                    if buf.remaining_mut() == 0 {
                        if buf.capacity() >= LINE_CAP {
                            anyhow::bail!("IMAP response line exceeded maximum length of {} bytes", LINE_CAP);
                        }
                        let add = GROW_STEP.min(LINE_CAP - buf.capacity());
                        buf.reserve(add);
                    }
                }
                Some(msg) = cmd_rx.recv() => {
                    if active.is_none() {
                        stream.write_all(msg.command.as_bytes()).await
                            .with_context(|| format!("Failed to send IMAP command: {}", msg.command))?;
                        stream.flush().await
                            .with_context(|| format!("Failed to flush IMAP command: {}", msg.command))?;
                        active = Some(ActiveCommand { tag: msg.tag, responder: msg.responder, collected: Vec::new() });
                    } else {
                        queue.push_back(msg);
                    }
                }
                else => break,
            }
        }
        Ok(())
    }
}

fn is_tagged_completion(line: &Bytes, tag: &str) -> bool {
    // Tagged completion is: <tag> SP (OK|NO|BAD) ... CRLF
    if line.len() < tag.len() + 4 {
        return false;
    }
    if &line[..tag.len()] != tag.as_bytes() {
        return false;
    }
    if line.get(tag.len()) != Some(&b' ') {
        return false;
    }
    // We accept any status for completion detection
    true
}

impl Client<ConnectedState> {
    #[tracing::instrument(skip(self, pass))]
    pub async fn login(self, user: &str, pass: &str) -> Result<Client<AuthenticatedState>> {
        tracing::info!("Attempting IMAP login");

        let tag = next_tag();
        let cmd = CommandBuilder::new(&tag)
            .login()
            .username(user)
            .password(pass)
            .as_string();

        let (tx, rx) = oneshot::channel::<Vec<Bytes>>();
        self.cmd_tx
            .send(CommandMessage {
                tag: tag.clone(),
                command: cmd,
                responder: tx,
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send login command: {}", e))?;

        let lines = rx.await.context("Login command timed out")?;
        tracing::debug!("Login response lines: {}", lines.len());

        // Basic status check: last tagged completion should be OK
        if let Some(last) = lines.iter().rev().find(|l| l.starts_with(tag.as_bytes())) {
            if !last.windows(3).any(|w| w == b" OK") {
                anyhow::bail!("Login failed: {}", String::from_utf8_lossy(last));
            }
        }

        Ok(Client::<AuthenticatedState> {
            cmd_tx: self.cmd_tx,
            unsol_rx: self.unsol_rx,
            _state: PhantomData,
        })
    }
}

impl Client<AuthenticatedState> {
    pub async fn fetch(&mut self, mailbox: &str, id: u32) -> Result<Vec<Envelope>> {
        // Select mailbox (simple approach; future: cache selected mailbox)
        let sel_tag = next_tag();
        let select_cmd = CommandBuilder::new(&sel_tag).select(mailbox).as_string();
        let (sel_tx, sel_rx) = oneshot::channel::<Vec<Bytes>>();
        self.cmd_tx
            .send(CommandMessage {
                tag: sel_tag.clone(),
                command: select_cmd,
                responder: sel_tx,
            })
            .await
            .context("Failed to send SELECT command")?;
        let sel_lines = sel_rx.await.context("SELECT timed out")?;
        if let Some(last) = sel_lines
            .iter()
            .rev()
            .find(|l| l.starts_with(sel_tag.as_bytes()))
        {
            if !last.windows(3).any(|w| w == b" OK") {
                anyhow::bail!("SELECT failed: {}", String::from_utf8_lossy(last));
            }
        }

        // Build FETCH 1:id ENVELOPE for subjects
        let set = SequenceSet::new().add_range(SequenceBound::Number(1), SequenceBound::Number(id));
        let fetch_tag = next_tag();
        let fetch_cmd = CommandBuilder::new(&fetch_tag)
            .fetch(set)
            .add_item(imap::commands::FetchItem::Envelope)
            .as_string();
        let (tx, rx) = oneshot::channel::<Vec<Bytes>>();
        self.cmd_tx
            .send(CommandMessage {
                tag: fetch_tag.clone(),
                command: fetch_cmd,
                responder: tx,
            })
            .await
            .context("Failed to send FETCH command")?;
        let lines = rx.await.context("FETCH timed out")?;

        let mut joined = BytesMut::new();
        for l in &lines {
            joined.extend_from_slice(l);
        }
        let mut envelopes = Vec::new();
        for (_num, data) in fetch::fetch_envelopes(&joined) {
            if let FetchData::Envelope(env) = data {
                envelopes.push(env);
            }
        }

        Ok(envelopes)
    }
}
