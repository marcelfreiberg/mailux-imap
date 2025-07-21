// use std::env;
// use imap::blocking::Builder;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();

//     let email = env::var("IMAP_EMAIL")?;
//     let password = env::var("IMAP_PASSWORD")?;
//     let imap_server = env::var("IMAP_SERVER")?;

//     // let client = imap::connect_tls(&imap_server)?;
//     // let mut client = Builder::new(&imap_server).tls().connect()?;
//     let client = Builder::new(&imap_server).tls().build().connect()?;

//     let mut session = client.login(&email, &password)?;

//     let mut msgs = session.fetch("INBOX", 2)?;
//     // let mut msgs = session.fetch("INBOX", "1:5")?;
//     // let mut msgs = session.fetch("INBOX", &[1, 2, 4, 9])?;

//     // while let Some(msg) = msgs.try_next().await()? {
//     while let Some(msg) = msgs.try_next()? {
//         println!("Subject {}", msg.subject());
//     }
    
//     Ok(())
// }

use std::env;
use imap::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let email = env::var("IMAP_EMAIL")?;
    let password = env::var("IMAP_PASSWORD")?;
    let imap_server = env::var("IMAP_SERVER")?;

    // let client = imap::connect_tls(&imap_server)?;
    // let mut client = Builder::new(&imap_server).tls().connect()?;
    let client = Builder::new(&imap_server).tls().build().connect().await?;

    let mut session = client.login(&email, &password).await?;

    let mut msgs = session.fetch("INBOX", 2).await?;
    // let mut msgs = session.fetch("INBOX", "1:5")?;
    // let mut msgs = session.fetch("INBOX", &[1, 2, 4, 9])?;

    // while let Some(msg) = msgs.try_next().await()? {
    while let Some(msg) = msgs.try_next()? {
        println!("Subject {}", msg.subject());
    }
    
    Ok(())
}