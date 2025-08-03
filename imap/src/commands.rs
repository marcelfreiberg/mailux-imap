use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

static TAG_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_tag() -> String {
    let tag_num = TAG_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("A{:04}", tag_num)
}

// For testing
pub fn reset_tag_counter() {
    TAG_COUNTER.store(1, Ordering::SeqCst);
}

pub trait Command {
    fn to_bytes(&self) -> Vec<u8>;
    fn tag(&self) -> &str;
}

pub struct LoginCommandBuilder {
    tag: String,
    username: String,
    password: String,
}

impl LoginCommandBuilder {
    pub fn new() -> Self {
        Self {
            tag: next_tag(),
            username: String::new(),
            password: String::new(),
        }
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = tag.to_string();
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }

    pub fn build(self) -> LoginCommand {
        LoginCommand {
            tag: self.tag,
            username: self.username,
            password: self.password,
        }
    }
}

pub struct LoginCommand {
    tag: String,
    username: String,
    password: String,
}

impl Command for LoginCommand {
    fn to_bytes(&self) -> Vec<u8> {
        format!("{} LOGIN {} {}", self.tag, self.username, self.password)
            .as_bytes()
            .to_vec()
    }

    fn tag(&self) -> &str {
        &self.tag
    }
}

impl fmt::Display for LoginCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} LOGIN {} {}", self.tag, self.username, self.password)
    }
}
