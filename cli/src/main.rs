use imap::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Builder::new("imap.example.com:993").build()?;
    // let client = Builder::new("imap.example.com:993").tls().build()?; <-- default
    // let client = Builder::new("imap.example.com:993").starttls().build()?;
    // let client = Builder::new("imap.example.com:993").plain().build()?;

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
