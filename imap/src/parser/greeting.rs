use super::{ParserError, Status, parse_status};
use nom::{
    IResult, Offset, Parser,
    bytes::streaming::{tag, take_until},
    character::streaming::crlf,
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
            separated_pair(parse_status, tag(" "), take_until("\r\n")),
            crlf,
        )
        .map(|(status, text)| Greeting { status, text }),
    )
    .parse(i)
}
