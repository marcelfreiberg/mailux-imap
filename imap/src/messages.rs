use crate::error::ImapError;

pub struct Message {
    subject: String,
}

pub struct Messages {
    messages: Vec<Message>,
}

impl Message {
    pub fn new(subject: String) -> Self {
        Self { subject }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl Messages {
    pub fn new(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    pub fn try_next(&mut self) -> Result<Option<Message>, ImapError> {
        if !self.messages.is_empty() {
            let result = self.messages.remove(0);
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
