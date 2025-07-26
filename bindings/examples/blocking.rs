#[cfg(feature = "blocking")]
use std::env;

#[cfg(feature = "blocking")]
use bindings::blocking::Builder;

#[cfg(feature = "blocking")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let email = env::var("IMAP_EMAIL")?;
    let password = env::var("IMAP_PASSWORD")?;
    let imap_server = env::var("IMAP_SERVER")?;

    let client = Builder::new(&imap_server).tls().build().connect()?;

    let mut session = client.login(&email, &password)?;

    let mut msgs = session.fetch("INBOX", 2)?;

    while let Some(msg) = msgs.try_next()? {
        println!("Subject: {}", msg.subject());
    }

    Ok(())
}

#[cfg(not(feature = "blocking"))]
fn main() {
    println!("This example requires the 'blocking' feature to be enabled.");
    println!("Run with: cargo run --example blocking --features blocking");
}
