use bindings::Builder;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let email = env::var("IMAP_EMAIL")?;
    let password = env::var("IMAP_PASSWORD")?;
    let imap_server = env::var("IMAP_SERVER")?;

    let client = Builder::new(&imap_server).tls().build().connect().await?;

    let mut session = client.login(&email, &password).await?;

    let mut msgs = session.fetch("INBOX", 2).await?;

    while let Some(msg) = msgs.try_next()? {
        println!("Subject: {}", msg.subject());
    }

    Ok(())
}
