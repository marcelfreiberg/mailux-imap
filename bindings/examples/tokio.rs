use bindings::Builder;
use std::env;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let email = env::var("IMAP_EMAIL")?;
    let password = env::var("IMAP_PASSWORD")?;
    let imap_server = env::var("IMAP_SERVER")?;
    let mailbox = env::var("IMAP_MAILBOX").unwrap_or_else(|_| "INBOX".to_string());
    let count: u32 = env::var("IMAP_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    println!("Connecting to {} ...", imap_server);
    let t0 = Instant::now();
    let client = Builder::new(&imap_server).tls().build().connect().await?;
    println!("Connected in {:.2?}", t0.elapsed());

    println!("Logging in as {} ...", email);
    let t1 = Instant::now();
    let mut session = client.login(&email, &password).await?;
    println!("Authenticated in {:.2?}", t1.elapsed());

    println!(
        "Fetching first {} subjects from mailbox {} ...",
        count, mailbox
    );
    let t2 = Instant::now();
    let mut msgs = session.fetch(&mailbox, count).await?;

    let mut idx = 1u32;
    while let Some(msg) = msgs.try_next()? {
        println!("#{:02}  {}", idx, msg.subject());
        idx += 1;
    }
    println!("Fetched in {:.2?}", t2.elapsed());

    Ok(())
}
