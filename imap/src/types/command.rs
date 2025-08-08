use crate::format::quote_astring;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum SequenceBound {
    Number(u32),
    Star,
}

#[derive(Debug, Clone)]
pub enum SequenceRange {
    Single(SequenceBound),
    Range(SequenceBound, SequenceBound),
}

#[derive(Debug, Clone, Default)]
pub struct SequenceSet {
    pub(crate) parts: Vec<SequenceRange>,
}

impl SequenceSet {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }
    pub fn all() -> Self {
        Self {
            parts: vec![SequenceRange::Single(SequenceBound::Star)],
        }
    }
    pub fn add_single(mut self, n: u32) -> Self {
        self.parts
            .push(SequenceRange::Single(SequenceBound::Number(n)));
        self
    }
    pub fn add_star(mut self) -> Self {
        self.parts.push(SequenceRange::Single(SequenceBound::Star));
        self
    }
    pub fn add_range(mut self, start: SequenceBound, end: SequenceBound) -> Self {
        self.parts.push(SequenceRange::Range(start, end));
        self
    }
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

impl Display for SequenceBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SequenceBound::Number(n) => write!(f, "{}", n),
            SequenceBound::Star => f.write_str("*"),
        }
    }
}

impl Display for SequenceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SequenceRange::Single(b) => write!(f, "{}", b),
            SequenceRange::Range(s, e) => write!(f, "{}:{}", s, e),
        }
    }
}

impl Display for SequenceSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for part in &self.parts {
            if !first {
                f.write_str(",")?;
            } else {
                first = false;
            }
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum StatusItem {
    Messages,
    Recent,
    UidNext,
    UidValidity,
    Unseen,
}

impl Display for StatusItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusItem::Messages => f.write_str("MESSAGES"),
            StatusItem::Recent => f.write_str("RECENT"),
            StatusItem::UidNext => f.write_str("UIDNEXT"),
            StatusItem::UidValidity => f.write_str("UIDVALIDITY"),
            StatusItem::Unseen => f.write_str("UNSEEN"),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SearchKey {
    All,
    Answered,
    Bcc(String),
    Before(String),
    Body(String),
    Cc(String),
    Deleted,
    Draft,
    Flagged,
    From(String),
    Header { name: String, value: String },
    Keyword(String),
    Larger(u32),
    New,
    Not(Box<SearchKey>),
    Old,
    On(String),
    Or(Box<SearchKey>, Box<SearchKey>),
    Recent,
    Seen,
    SentBefore(String),
    SentOn(String),
    SentSince(String),
    Since(String),
    Smaller(u32),
    Subject(String),
    Text(String),
    To(String),
    Unanswered,
    Undeleted,
    Undraft,
    Unflagged,
    Unkeyword(String),
    Unseen,
    Uid(SequenceSet),
}

impl Display for SearchKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SearchKey as K;
        match self {
            K::All => f.write_str("ALL"),
            K::Answered => f.write_str("ANSWERED"),
            K::Bcc(s) => write!(f, "BCC {}", quote_astring(s)),
            K::Before(s) => write!(f, "BEFORE {}", s),
            K::Body(s) => write!(f, "BODY {}", quote_astring(s)),
            K::Cc(s) => write!(f, "CC {}", quote_astring(s)),
            K::Deleted => f.write_str("DELETED"),
            K::Draft => f.write_str("DRAFT"),
            K::Flagged => f.write_str("FLAGGED"),
            K::From(s) => write!(f, "FROM {}", quote_astring(s)),
            K::Header { name, value } => {
                write!(f, "HEADER {} {}", quote_astring(name), quote_astring(value))
            }
            K::Keyword(s) => write!(f, "KEYWORD {}", s),
            K::Larger(n) => write!(f, "LARGER {}", n),
            K::New => f.write_str("NEW"),
            K::Not(k) => write!(f, "NOT ({})", k),
            K::Old => f.write_str("OLD"),
            K::On(s) => write!(f, "ON {}", s),
            K::Or(a, b) => write!(f, "OR ({}) ({})", a, b),
            K::Recent => f.write_str("RECENT"),
            K::Seen => f.write_str("SEEN"),
            K::SentBefore(s) => write!(f, "SENTBEFORE {}", s),
            K::SentOn(s) => write!(f, "SENTON {}", s),
            K::SentSince(s) => write!(f, "SENTSINCE {}", s),
            K::Since(s) => write!(f, "SINCE {}", s),
            K::Smaller(n) => write!(f, "SMALLER {}", n),
            K::Subject(s) => write!(f, "SUBJECT {}", quote_astring(s)),
            K::Text(s) => write!(f, "TEXT {}", quote_astring(s)),
            K::To(s) => write!(f, "TO {}", quote_astring(s)),
            K::Unanswered => f.write_str("UNANSWERED"),
            K::Undeleted => f.write_str("UNDELETED"),
            K::Undraft => f.write_str("UNDRAFT"),
            K::Unflagged => f.write_str("UNFLAGGED"),
            K::Unkeyword(s) => write!(f, "UNKEYWORD {}", s),
            K::Unseen => f.write_str("UNSEEN"),
            K::Uid(set) => write!(f, "UID {}", set),
        }
    }
}
