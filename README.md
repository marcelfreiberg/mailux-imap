# rust-imap

A modern IMAP client library for Rust with both async and blocking support.

> **⚠️ Note**: This repository name is generic and likely to change. The project is under heavy development.

## Quick Start

```rust
let client = imap::connect_tls("imap.server.com:993").await?;

let mut session = client.login("user@example.com", "password").await?;

let messages = session.fetch("INBOX", 1).await?;
```
