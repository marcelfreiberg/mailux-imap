use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = env::var("IMAP_EMAIL")?;
    let password = env::var("IMAP_PASSWORD")?;
    let imap_server = env::var("IMAP_SERVER")?;

    let client = imap::connect_tls(&imap_server)?;
    // let mut client = imap::Builder::new("imap.example.com:993").tls().connect()?;
    // let mut client = imap::Builder::new("imap.example.com:993").tls().build().connect()?;

    let mut session = client.login(&email, &password)?;

    let mut msgs = session.fetch("INBOX", 2)?;
    // let mut msgs = session.fetch("INBOX", "1:5")?;
    // let mut msgs = session.fetch("INBOX", &[1, 2, 4, 9])?;

    // while let Some(msg) = msgs.try_next().await()? {
    while let Some(msg) = msgs.try_next()? {
        println!("Subject {}", msg.subject());
    }
    
    Ok(())
}
