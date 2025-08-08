use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Flag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    Keyword(String),
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::Seen => f.write_str("\\Seen"),
            Flag::Answered => f.write_str("\\Answered"),
            Flag::Flagged => f.write_str("\\Flagged"),
            Flag::Deleted => f.write_str("\\Deleted"),
            Flag::Draft => f.write_str("\\Draft"),
            Flag::Recent => f.write_str("\\Recent"),
            Flag::Keyword(k) => f.write_str(k),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Ok,
    No,
    Bad,
}
