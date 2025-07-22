use nom::{
    IResult, Offset, Parser,
    bytes::streaming::{tag, tag_no_case},
    character::streaming::{crlf, not_line_ending, space1},
    combinator::map,
    sequence::{delimited, separated_pair},
};
use std::borrow::Cow;
use super::{ParserError, Status};

#[derive(Debug, Clone)]
pub struct Greeting<'a> {
    pub status: Status,
    pub text: Cow<'a, [u8]>,
}

pub fn try_parse_greeting(buf: &[u8]) -> Result<Option<(Greeting, usize)>, ParserError> {
    match parse_greeting(buf) {
        Ok((remaining, greeting)) => Ok(Some((greeting, buf.offset(remaining)))),
        Err(nom::Err::Incomplete(_)) => Err(ParserError::Incomplete),
        Err(_) => Err(ParserError::InvalidResponse),
    }
}

fn parse_greeting(i: &[u8]) -> IResult<&[u8], Greeting<'_>> {
    map(
        delimited(
            tag("* "),
            separated_pair(parse_greeting_status, space1, not_line_ending),
            crlf,
        ),
        |(status, text)| Greeting { status, text: Cow::Borrowed(text) },
    ).parse(i)
}

fn parse_greeting_status(i: &[u8]) -> IResult<&[u8], Status> {
    nom::branch::alt((
        map(tag_no_case("OK"), |_| Status::Ok),
        map(tag_no_case("PREAUTH"), |_| Status::PreAuth),
        map(tag_no_case("BYE"), |_| Status::Bye),
    ))
    .parse(i)
} 