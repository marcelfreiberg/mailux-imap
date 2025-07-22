use nom::{
    IResult, Offset, Parser,
    bytes::streaming::{tag, take_until},
    character::streaming::{crlf, space1},
    combinator::map,
    sequence::{separated_pair, terminated},
};
use std::borrow::Cow;
use super::{ParserError, Status, Response};

pub fn try_parse_tagged_response(buf: &[u8]) -> Result<Option<(Response<'static>, usize)>, ParserError> {
    match parse_tagged_response(buf) {
        Ok((remaining, response)) => Ok(Some((response, buf.offset(remaining)))),
        Err(nom::Err::Incomplete(_)) => Err(ParserError::Incomplete),
        Err(_) => Err(ParserError::InvalidResponse),
    }
}

fn parse_tagged_response(i: &[u8]) -> IResult<&[u8], Response<'static>> {
    map(
        terminated(
            separated_pair(
                take_until(" "),
                space1,
                separated_pair(parse_status, space1, take_until("\r\n")),
            ),
            crlf,
        ),
        |(tag, (status, text))| Response::Tagged {
            tag: Cow::Owned(tag.to_vec()),
            status,
            text: Cow::Owned(text.to_vec()),
        },
    ).parse(i)
}

fn parse_status(i: &[u8]) -> IResult<&[u8], Status> {
    nom::branch::alt((
        map(tag("OK"), |_| Status::Ok),
        map(tag("NO"), |_| Status::No),
        map(tag("BAD"), |_| Status::Bad),
    ))
    .parse(i)
} 