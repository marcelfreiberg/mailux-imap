use std::borrow::Cow;
use thiserror::Error;

pub mod auth;
pub mod greeting;
pub mod mailbox;

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
    Tagged {
        tag: Cow<'a, [u8]>,
        status: Status,
        text: Cow<'a, [u8]>,
    },
    Untagged {
        status: Status,
        text: Cow<'a, [u8]>,
    },
}
