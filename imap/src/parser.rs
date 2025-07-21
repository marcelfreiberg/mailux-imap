use bytes::{Bytes, BytesMut};
use nom::{
    IResult, Parser, Offset,
    branch::alt,
    bytes::streaming::{tag, take_while1},
    character::streaming::crlf,
    combinator::map,
    sequence::{preceded, separated_pair, terminated},
};
use thiserror::Error;

// RFC 3501 § 7.1 - Generic over any byte container
#[derive(Debug, Clone)]
pub enum Status<B = Bytes> {
    // Can be tagged or untagged
    Ok,
    No,
    Bad,
    // Always untagged
    PreAuth,
    Bye,
    // Any extension status
    Unknown(B),
}

#[derive(Debug, Clone)]
pub enum Response<B = Bytes> {
    Untagged {
        status: Status<B>,
        text: B,
    },
    Tagged {
        tag: B,
        status: Status<B>,
        text: B,
    },
    Continuation {
        text: B,
    },
}

// Type alias for owned responses
pub type OwnedResponse = Response<Bytes>;

pub trait ToBytes {
    fn to_bytes(self) -> Bytes;
}

impl ToBytes for Bytes {
    fn to_bytes(self) -> Bytes {
        self // No copy needed - already Bytes
    }
}

impl ToBytes for &[u8] {
    fn to_bytes(self) -> Bytes {
        Bytes::copy_from_slice(self)
    }
}

fn to_bytes<B: ToBytes>(b: B) -> Bytes {
    b.to_bytes()
}

impl<B: Clone + AsRef<[u8]> + ToBytes> Response<B> {
    pub fn into_owned(self) -> Response<Bytes> {
        match self {
            Response::Untagged { status, text } => Response::Untagged {
                status: status.into_owned(),
                text: to_bytes(text),
            },
            Response::Tagged { tag, status, text } => Response::Tagged {
                tag: to_bytes(tag),
                status: status.into_owned(),
                text: to_bytes(text),
            },
            Response::Continuation { text } => Response::Continuation {
                text: to_bytes(text),
            },
        }
    }
}

impl<B: Clone + AsRef<[u8]> + ToBytes> Status<B> {
    pub fn into_owned(self) -> Status<Bytes> {
        match self {
            Status::Ok => Status::Ok,
            Status::No => Status::No,
            Status::Bad => Status::Bad,
            Status::PreAuth => Status::PreAuth,
            Status::Bye => Status::Bye,
            Status::Unknown(b) => Status::Unknown(to_bytes(b)),
        }
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Need more data")]
    Incomplete,
    #[error("Invalid IMAP response")]
    InvalidResponse,
}

pub fn decode(buf: &BytesMut) -> Result<Option<(OwnedResponse, usize)>, ParserError> {
    let input = &buf[..];
    match response(input) {
        Ok((rest, resp)) => {
            let consumed = input.offset(rest);
            let owned_resp = resp.into_owned();
            Ok(Some((owned_resp, consumed)))
        }
        Err(nom::Err::Incomplete(_)) => Ok(None),
        Err(_) => Err(ParserError::InvalidResponse),
    }
}

fn response(i: &[u8]) -> IResult<&[u8], Response<&[u8]>> {
    terminated(
        alt((tagged_response, untagged_response, continuation)),
        crlf,
    ).parse(i)
}

fn untagged_response(i: &[u8]) -> IResult<&[u8], Response<&[u8]>> {
    map(
        preceded(tag(&b"* "[..]), separated_pair(status, tag(&b" "[..]), text_line)),
        |(status, text)| Response::Untagged { status, text },
    ).parse(i)
}

fn tagged_response(i: &[u8]) -> IResult<&[u8], Response<&[u8]>> {
    map(
        (tag_token, tag(&b" "[..]), status, tag(&b" "[..]), text_line),
        |(tag, _, status, _, text)| Response::Tagged { tag, status, text },
    ).parse(i)
}

fn continuation(i: &[u8]) -> IResult<&[u8], Response<&[u8]>> {
    map(preceded(tag(&b"+ "[..]), text_line), |text| Response::Continuation {
        text,
    }).parse(i)
}

fn status(i: &[u8]) -> IResult<&[u8], Status<&[u8]>> {
    alt((
        map(tag(&b"OK"[..]), |_| Status::Ok),
        map(tag(&b"NO"[..]), |_| Status::No),
        map(tag(&b"BAD"[..]), |_| Status::Bad),
        map(tag(&b"PREAUTH"[..]), |_| Status::PreAuth),
        map(tag(&b"BYE"[..]), |_| Status::Bye),
        map(atom, |s| Status::Unknown(s)), // Raw bytes, no UTF-8 assumption
    )).parse(i)
}

fn text_line(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(|c: u8| c != b'\r' && c != b'\n')(i)
}

fn tag_token(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(|c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.')(i)
}

fn atom(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(|c: u8| c.is_ascii_alphanumeric() || b"!#$&'*+-/=?^_`|~".contains(&c))(i)
}
