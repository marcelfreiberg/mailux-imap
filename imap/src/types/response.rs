use super::common::{Flag, Status};

#[derive(Debug, Clone)]
pub enum Response {
    Tagged {
        tag: String,
        status: Status,
        code: Option<String>,
        text: String,
    },
    Untagged(UntaggedResponse),
    Continuation(Option<String>),
}

#[derive(Debug, Clone)]
pub enum UntaggedResponse {
    Exists(u32),
    Recent(u32),
    Expunge(u32),
    Flags(Vec<Flag>),
    Search(Vec<u32>),
    Fetch { seq: u32, data: FetchData },
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub subject: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FetchData {
    Envelope(Envelope),
    Flags(Vec<Flag>),
    InternalDate(String),
    Rfc822Size(u32),
    Uid(u32),
}
