use std::fmt::{self, Display, Write as _};

fn quote_astring(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('"');
    for ch in input.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[derive(Debug, Clone, Default)]
pub struct SequenceSet {
    parts: Vec<SequenceRange>,
}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

fn join_flags(flags: &[Flag]) -> String {
    let mut s = String::from("(");
    let mut first = true;
    for flag in flags {
        if !first {
            s.push(' ');
        } else {
            first = false;
        }
        let _ = write!(&mut s, "{}", flag);
    }
    s.push(')');
    s
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

fn join_status_items(items: &[StatusItem]) -> String {
    let mut s = String::from("(");
    let mut first = true;
    for it in items {
        if !first {
            s.push(' ');
        } else {
            first = false;
        }
        let _ = write!(&mut s, "{}", it);
    }
    s.push(')');
    s
}

#[derive(Debug, Clone)]
pub enum FetchItem {
    All,
    Fast,
    Full,
    Body,
    BodyPeek,
    BodySection(String),
    BodyPeekSection(String),
    Envelope,
    Flags,
    InternalDate,
    Rfc822,
    Rfc822Header,
    Rfc822Text,
    Rfc822Size,
    Uid,
}

impl Display for FetchItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchItem::All => f.write_str("ALL"),
            FetchItem::Fast => f.write_str("FAST"),
            FetchItem::Full => f.write_str("FULL"),
            FetchItem::Body => f.write_str("BODY"),
            FetchItem::BodyPeek => f.write_str("BODY.PEEK"),
            FetchItem::BodySection(sec) => write!(f, "BODY[{}]", sec),
            FetchItem::BodyPeekSection(sec) => write!(f, "BODY.PEEK[{}]", sec),
            FetchItem::Envelope => f.write_str("ENVELOPE"),
            FetchItem::Flags => f.write_str("FLAGS"),
            FetchItem::InternalDate => f.write_str("INTERNALDATE"),
            FetchItem::Rfc822 => f.write_str("RFC822"),
            FetchItem::Rfc822Header => f.write_str("RFC822.HEADER"),
            FetchItem::Rfc822Text => f.write_str("RFC822.TEXT"),
            FetchItem::Rfc822Size => f.write_str("RFC822.SIZE"),
            FetchItem::Uid => f.write_str("UID"),
        }
    }
}

fn join_fetch_items(items: &[FetchItem]) -> String {
    let mut s = String::from("(");
    let mut first = true;
    for it in items {
        if !first {
            s.push(' ');
        } else {
            first = false;
        }
        let _ = write!(&mut s, "{}", it);
    }
    s.push(')');
    s
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

fn join_search_keys(keys: &[SearchKey]) -> String {
    let mut s = String::new();
    let mut first = true;
    for k in keys {
        if !first {
            s.push(' ');
        } else {
            first = false;
        }
        let _ = write!(&mut s, "{}", k);
    }
    s
}

pub struct CommandBuilder {
    tag: String,
}

impl CommandBuilder {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }

    // Session
    pub fn capability(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "CAPABILITY")
    }
    pub fn noop(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "NOOP")
    }
    pub fn logout(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "LOGOUT")
    }
    pub fn starttls(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "STARTTLS")
    }

    // Auth
    pub fn authenticate(self, mechanism: &str) -> SimpleWithArg {
        SimpleWithArg::new(self.tag, "AUTHENTICATE", mechanism)
    }
    pub fn login(self) -> LoginCommandBuilder<NoUsername, NoPassword> {
        LoginCommandBuilder::new(&self.tag)
    }

    // Mailbox selection
    pub fn select(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "SELECT", mailbox)
    }
    pub fn examine(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "EXAMINE", mailbox)
    }

    // Mailbox management
    pub fn create(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "CREATE", mailbox)
    }
    pub fn delete(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "DELETE", mailbox)
    }
    pub fn rename(self, from: &str, to: &str) -> RenameCommand {
        RenameCommand::new(self.tag, from, to)
    }
    pub fn subscribe(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "SUBSCRIBE", mailbox)
    }
    pub fn unsubscribe(self, mailbox: &str) -> MailboxCommand {
        MailboxCommand::new(self.tag, "UNSUBSCRIBE", mailbox)
    }
    pub fn list(self, reference: &str, pattern: &str) -> ListCommand {
        ListCommand::new(self.tag, "LIST", reference, pattern)
    }
    pub fn lsub(self, reference: &str, pattern: &str) -> ListCommand {
        ListCommand::new(self.tag, "LSUB", reference, pattern)
    }
    pub fn status(self, mailbox: &str, items: Vec<StatusItem>) -> StatusCommand {
        StatusCommand::new(self.tag, mailbox, items)
    }

    // Message ops
    pub fn append(self, mailbox: &str) -> AppendCommandBuilder {
        AppendCommandBuilder::new(self.tag, mailbox)
    }
    pub fn check(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "CHECK")
    }
    pub fn close(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "CLOSE")
    }
    pub fn expunge(self) -> SimpleCommand {
        SimpleCommand::new(self.tag, "EXPUNGE")
    }

    pub fn search(self) -> SearchCommandBuilder {
        SearchCommandBuilder::new(self.tag, None)
    }
    pub fn fetch(self, set: SequenceSet) -> FetchCommandBuilder {
        FetchCommandBuilder::new(self.tag, false, set)
    }
    pub fn store(self, set: SequenceSet) -> StoreCommandBuilder {
        StoreCommandBuilder::new(self.tag, false, set)
    }
    pub fn copy(self, set: SequenceSet, mailbox: &str) -> CopyCommand {
        CopyCommand::new(self.tag, false, set, mailbox)
    }

    // UID scope
    pub fn uid(self) -> UidScope {
        UidScope { tag: self.tag }
    }
}

pub struct SimpleCommand {
    tag: String,
    name: &'static str,
}
impl SimpleCommand {
    fn new(tag: String, name: &'static str) -> Self {
        Self { tag, name }
    }
    pub fn as_string(&self) -> String {
        format!("{} {}\r\n", self.tag, self.name)
    }
}

pub struct SimpleWithArg {
    tag: String,
    name: &'static str,
    arg: String,
}
impl SimpleWithArg {
    fn new(tag: String, name: &'static str, arg: &str) -> Self {
        Self {
            tag,
            name,
            arg: arg.to_string(),
        }
    }
    pub fn as_string(&self) -> String {
        format!("{} {} {}\r\n", self.tag, self.name, self.arg)
    }
}

pub struct MailboxCommand {
    tag: String,
    name: &'static str,
    mailbox: String,
}
impl MailboxCommand {
    fn new(tag: String, name: &'static str, mailbox: &str) -> Self {
        Self {
            tag,
            name,
            mailbox: mailbox.to_string(),
        }
    }
    pub fn as_string(&self) -> String {
        format!(
            "{} {} {}\r\n",
            self.tag,
            self.name,
            quote_astring(&self.mailbox)
        )
    }
}

pub struct RenameCommand {
    tag: String,
    from: String,
    to: String,
}
impl RenameCommand {
    fn new(tag: String, from: &str, to: &str) -> Self {
        Self {
            tag,
            from: from.to_string(),
            to: to.to_string(),
        }
    }
    pub fn as_string(&self) -> String {
        format!(
            "{} RENAME {} {}\r\n",
            self.tag,
            quote_astring(&self.from),
            quote_astring(&self.to)
        )
    }
}

pub struct ListCommand {
    tag: String,
    name: &'static str,
    reference: String,
    pattern: String,
}
impl ListCommand {
    fn new(tag: String, name: &'static str, reference: &str, pattern: &str) -> Self {
        Self {
            tag,
            name,
            reference: reference.to_string(),
            pattern: pattern.to_string(),
        }
    }
    pub fn as_string(&self) -> String {
        format!(
            "{} {} {} {}\r\n",
            self.tag,
            self.name,
            quote_astring(&self.reference),
            quote_astring(&self.pattern)
        )
    }
}

pub struct StatusCommand {
    tag: String,
    mailbox: String,
    items: Vec<StatusItem>,
}
impl StatusCommand {
    fn new(tag: String, mailbox: &str, items: Vec<StatusItem>) -> Self {
        Self {
            tag,
            mailbox: mailbox.to_string(),
            items,
        }
    }
    pub fn as_string(&self) -> String {
        format!(
            "{} STATUS {} {}\r\n",
            self.tag,
            quote_astring(&self.mailbox),
            join_status_items(&self.items)
        )
    }
}

pub struct AppendCommandBuilder {
    tag: String,
    mailbox: String,
    flags: Vec<Flag>,
    internal_date: Option<String>,
    literal_len: Option<usize>,
    literal: Option<Vec<u8>>,
}
impl AppendCommandBuilder {
    fn new(tag: String, mailbox: &str) -> Self {
        Self {
            tag,
            mailbox: mailbox.to_string(),
            flags: Vec::new(),
            internal_date: None,
            literal_len: None,
            literal: None,
        }
    }
    pub fn flags(mut self, flags: Vec<Flag>) -> Self {
        self.flags = flags;
        self
    }
    pub fn internal_date(mut self, rfc822_date_time: &str) -> Self {
        self.internal_date = Some(rfc822_date_time.to_string());
        self
    }
    pub fn literal(mut self, bytes: Vec<u8>) -> Self {
        self.literal_len = Some(bytes.len());
        self.literal = Some(bytes);
        self
    }
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        let _ = write!(
            &mut s,
            "{} APPEND {}",
            self.tag,
            quote_astring(&self.mailbox)
        );
        if !self.flags.is_empty() {
            s.push(' ');
            s.push_str(&join_flags(&self.flags));
        }
        if let Some(date) = &self.internal_date {
            let _ = write!(&mut s, " {}", quote_astring(date));
        }
        if let Some(n) = self.literal_len {
            let _ = write!(&mut s, " {{{}}}\r\n", n);
        } else {
            s.push_str("\r\n");
        }
        s
    }
    pub fn literal_bytes(&self) -> Option<&[u8]> {
        self.literal.as_deref()
    }
}

pub struct SearchCommandBuilder {
    tag: String,
    charset: Option<String>,
    keys: Vec<SearchKey>,
    uid: bool,
}
impl SearchCommandBuilder {
    fn new(tag: String, charset: Option<String>) -> Self {
        Self {
            tag,
            charset,
            keys: Vec::new(),
            uid: false,
        }
    }
    pub fn charset(mut self, charset: &str) -> Self {
        self.charset = Some(charset.to_string());
        self
    }
    pub fn key(mut self, key: SearchKey) -> Self {
        self.keys.push(key);
        self
    }
    pub fn keys(mut self, keys: Vec<SearchKey>) -> Self {
        self.keys.extend(keys);
        self
    }
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        let cmd = if self.uid { "UID SEARCH" } else { "SEARCH" };
        let _ = write!(&mut s, "{} {}", self.tag, cmd);
        if let Some(cs) = &self.charset {
            let _ = write!(&mut s, " CHARSET {}", cs);
        }
        if !self.keys.is_empty() {
            let _ = write!(&mut s, " {}", join_search_keys(&self.keys));
        }
        s.push_str("\r\n");
        s
    }
}

pub struct FetchCommandBuilder {
    tag: String,
    uid: bool,
    set: SequenceSet,
    items: Vec<FetchItem>,
}
impl FetchCommandBuilder {
    fn new(tag: String, uid: bool, set: SequenceSet) -> Self {
        Self {
            tag,
            uid,
            set,
            items: Vec::new(),
        }
    }
    pub fn items(mut self, items: Vec<FetchItem>) -> Self {
        self.items = items;
        self
    }
    pub fn add_item(mut self, item: FetchItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        let cmd = if self.uid { "UID FETCH" } else { "FETCH" };
        let _ = write!(&mut s, "{} {} {}", self.tag, cmd, self.set);
        if !self.items.is_empty() {
            let _ = write!(&mut s, " {}", join_fetch_items(&self.items));
        }
        s.push_str("\r\n");
        s
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StoreAction {
    Replace,
    Add,
    Remove,
}

pub struct StoreCommandBuilder {
    tag: String,
    uid: bool,
    set: SequenceSet,
    action: StoreAction,
    silent: bool,
    flags: Vec<Flag>,
}
impl StoreCommandBuilder {
    fn new(tag: String, uid: bool, set: SequenceSet) -> Self {
        Self {
            tag,
            uid,
            set,
            action: StoreAction::Replace,
            silent: false,
            flags: Vec::new(),
        }
    }
    pub fn replace(mut self) -> Self {
        self.action = StoreAction::Replace;
        self
    }
    pub fn add(mut self) -> Self {
        self.action = StoreAction::Add;
        self
    }
    pub fn remove(mut self) -> Self {
        self.action = StoreAction::Remove;
        self
    }
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
    pub fn flags(mut self, flags: Vec<Flag>) -> Self {
        self.flags = flags;
        self
    }
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        let cmd = if self.uid { "UID STORE" } else { "STORE" };
        let _ = write!(&mut s, "{} {} {} ", self.tag, cmd, self.set);
        match (self.action, self.silent) {
            (StoreAction::Replace, false) => s.push_str("FLAGS "),
            (StoreAction::Replace, true) => s.push_str("FLAGS.SILENT "),
            (StoreAction::Add, false) => s.push_str("+FLAGS "),
            (StoreAction::Add, true) => s.push_str("+FLAGS.SILENT "),
            (StoreAction::Remove, false) => s.push_str("-FLAGS "),
            (StoreAction::Remove, true) => s.push_str("-FLAGS.SILENT "),
        }
        s.push_str(&join_flags(&self.flags));
        s.push_str("\r\n");
        s
    }
}

pub struct CopyCommand {
    tag: String,
    uid: bool,
    set: SequenceSet,
    mailbox: String,
}
impl CopyCommand {
    fn new(tag: String, uid: bool, set: SequenceSet, mailbox: &str) -> Self {
        Self {
            tag,
            uid,
            set,
            mailbox: mailbox.to_string(),
        }
    }
    pub fn as_string(&self) -> String {
        let cmd = if self.uid { "UID COPY" } else { "COPY" };
        format!(
            "{} {} {} {}\r\n",
            self.tag,
            cmd,
            self.set,
            quote_astring(&self.mailbox)
        )
    }
}

pub struct UidScope {
    tag: String,
}
impl UidScope {
    pub fn search(self) -> SearchCommandBuilder {
        let mut b = SearchCommandBuilder::new(self.tag, None);
        b.uid = true;
        b
    }
    pub fn fetch(self, set: SequenceSet) -> FetchCommandBuilder {
        FetchCommandBuilder::new(self.tag, true, set)
    }
    pub fn store(self, set: SequenceSet) -> StoreCommandBuilder {
        StoreCommandBuilder::new(self.tag, true, set)
    }
    pub fn copy(self, set: SequenceSet, mailbox: &str) -> CopyCommand {
        CopyCommand::new(self.tag, true, set, mailbox)
    }
}

pub struct NoUsername;
pub struct HasUsername(String);
pub struct NoPassword;
pub struct HasPassword(String);

pub struct LoginCommandBuilder<U = NoUsername, P = NoPassword> {
    tag: String,
    username: U,
    password: P,
}

impl LoginCommandBuilder<NoUsername, NoPassword> {
    fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            username: NoUsername,
            password: NoPassword,
        }
    }
}

impl<P> LoginCommandBuilder<NoUsername, P> {
    pub fn username(self, username: &str) -> LoginCommandBuilder<HasUsername, P> {
        LoginCommandBuilder {
            tag: self.tag,
            username: HasUsername(username.to_string()),
            password: self.password,
        }
    }
}

impl<U> LoginCommandBuilder<U, NoPassword> {
    pub fn password(self, password: &str) -> LoginCommandBuilder<U, HasPassword> {
        LoginCommandBuilder {
            tag: self.tag,
            username: self.username,
            password: HasPassword(password.to_string()),
        }
    }
}

impl LoginCommandBuilder<HasUsername, HasPassword> {
    pub fn as_string(&self) -> String {
        format!(
            "{} LOGIN {} {}\r\n",
            self.tag,
            quote_astring(&self.username.0),
            quote_astring(&self.password.0)
        )
    }
}
