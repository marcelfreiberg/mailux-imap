use super::{ParserError, Response, parse_status};
use nom::{
    IResult, Offset, Parser,
    bytes::streaming::take_until,
    character::streaming::{crlf, space1},
    combinator::map,
    sequence::{separated_pair, terminated},
};

pub fn try_parse_tagged_response(buf: &[u8]) -> Result<Option<(Response<'_>, usize)>, ParserError> {
    match parse_tagged_response(buf) {
        Ok((remaining, response)) => Ok(Some((response, buf.offset(remaining)))),
        Err(nom::Err::Incomplete(_)) => Err(ParserError::Incomplete),
        Err(_) => Err(ParserError::InvalidResponse),
    }
}

fn parse_tagged_response(i: &[u8]) -> IResult<&[u8], Response<'_>> {
    map(
        terminated(
            separated_pair(
                take_until(" "),
                space1,
                separated_pair(parse_status, space1, take_until("\r\n")),
            ),
            crlf,
        ),
        |(tag, (status, text))| Response::Tagged { tag, status, text },
    )
    .parse(i)
}
