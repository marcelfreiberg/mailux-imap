fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = imap::connect_tls("imap.example.com:993")?;
    
    // Or use the builder pattern with direct connect:
    // let mut client = imap::Builder::new("imap.example.com:993").tls().connect()?;
    
    // Or use the full explicit builder pattern:
    // let mut client = imap::Builder::new("imap.example.com:993").tls().build().connect()?;

    let mut session = client.login("user", "pw")?;

    let mut msgs = session.fetch("INBOX", 2)?;
    // let mut msgs = session.fetch("INBOX", "1:5")?;
    // let mut msgs = session.fetch("INBOX", &[1, 2, 4, 9])?;

    // while let Some(msg) = msgs.try_next().await()? {
    while let Some(msg) = msgs.try_next()? {
        println!("Subject {}", msg.subject());
    }
    
    Ok(())
}
