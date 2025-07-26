use std::borrow::Cow;
use thiserror::Error;

pub mod greeting;
pub mod auth;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Need more data")]
    Incomplete,
    #[error("Invalid IMAP response")]
    InvalidResponse,
}

#[derive(Debug, Clone)]
pub enum Status {
    Ok,
    No,
    Bad,
    PreAuth,
    Bye,
}

#[derive(Debug, Clone)]
pub enum Response<'a> {
    Greeting(greeting::Greeting<'a>),
    Tagged { tag: Cow<'a, [u8]>, status: Status, text: Cow<'a, [u8]> },
    Untagged { status: Status, text: Cow<'a, [u8]> },
}

pub type OwnedResponse = Response<'static>;

pub fn try_parse_response(buf: &[u8]) -> Result<Option<(OwnedResponse, usize)>, ParserError> {
    if buf.starts_with(b"* ") {
        // Try greeting first
        if let Ok(Some((greeting, size))) = greeting::try_parse_greeting(buf) {
            let owned = Response::Greeting(greeting::Greeting {
                status: greeting.status,
                text: Cow::Owned(greeting.text.into_owned()),
            });
            return Ok(Some((owned, size)));
        }
        
        // Other untagged responses would go here
        Err(ParserError::InvalidResponse)
    } else {
        // Tagged responses
        auth::try_parse_tagged_response(buf)
            .map(|r| r.map(|(resp, size)| (resp, size)))
    }
} 