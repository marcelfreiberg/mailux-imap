use nom::{IResult, Parser, branch::alt, bytes::streaming::tag_no_case, combinator::value};
use thiserror::Error;

pub mod auth;
pub mod fetch;
pub mod greeting;

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
}

pub fn parse_status(i: &[u8]) -> IResult<&[u8], Status> {
    alt((
        value(Status::Ok, tag_no_case("OK")),
        value(Status::No, tag_no_case("NO")),
        value(Status::Bad, tag_no_case("BAD")),
    ))
    .parse(i)
}

#[derive(Debug, Clone)]
pub enum Response<'a> {
    Tagged {
        tag: &'a [u8],
        status: Status,
        text: &'a [u8],
    },
    Untagged {
        status: Status,
        text: &'a [u8],
    },
}
