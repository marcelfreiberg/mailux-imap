use super::{ParserError, Status};
use nom::{
    IResult, Offset, Parser,
    branch::alt,
    bytes::streaming::{tag, tag_no_case, take_until},
    character::streaming::crlf,
    combinator::value,
    sequence::{preceded, separated_pair, terminated},
};

#[derive(Debug, Clone)]
pub struct Greeting<'a> {
    pub status: Status,
    pub text: &'a [u8],
}

pub fn try_parse(buf: &[u8]) -> Result<Option<(Greeting, usize)>, ParserError> {
    match parse_greeting(buf) {
        Ok((remaining, greeting)) => Ok(Some((greeting, buf.offset(remaining)))),
        Err(nom::Err::Incomplete(_)) => Err(ParserError::Incomplete),
        Err(_) => Err(ParserError::InvalidResponse),
    }
}

fn parse_greeting(i: &[u8]) -> IResult<&[u8], Greeting<'_>> {
    preceded(
        tag("* "),
        terminated(
            separated_pair(parse_greeting_status, tag(" "), take_until("\r\n")),
            crlf,
        )
        .map(|(status, text)| Greeting { status, text }),
    )
    .parse(i)
}

fn parse_greeting_status(i: &[u8]) -> IResult<&[u8], Status> {
    alt((
        value(Status::Ok, tag_no_case("OK")),
        value(Status::PreAuth, tag_no_case("PREAUTH")),
        value(Status::Bye, tag_no_case("BYE")),
    ))
    .parse(i)
}
