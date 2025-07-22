use crate::ImapError;

pub struct Message {
    // TODO: make this field private again
    pub subject: String,
}

pub struct Messages {
    // TODO: make this field private again
    pub messages: Vec<Result<Message, ImapError>>,
}

impl Message {
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl Messages {
    pub fn try_next(&mut self) -> Result<Option<Message>, ImapError> {
        if !self.messages.is_empty() {
            let result = self.messages.remove(0);
            match result {
                Ok(message) => Ok(Some(message)),
                Err(error) => Err(error),
            }
        } else {
            Ok(None)
        }
    }
}
