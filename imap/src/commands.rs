pub struct CommandBuilder {
    tag: String,
}

impl CommandBuilder {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }

    pub fn login(self) -> LoginCommandBuilder<NoUsername, NoPassword> {
        LoginCommandBuilder::new(&self.tag)
    }

    // Future commands to add:
    // pub fn select(self) -> SelectCommandBuilder<NoMailbox> { ... }
    // pub fn fetch(self) -> FetchCommandBuilder<NoRange, NoItems> { ... }
    // pub fn search(self) -> SearchCommandBuilder<NoCriteria> { ... }
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
        format!("{} LOGIN {} {}", self.tag, self.username.0, self.password.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_command() {
        let command = CommandBuilder::new("A0001")
            .login()
            .username("testuser")
            .password("testpass");

        assert_eq!(command.as_string(), "A0001 LOGIN testuser testpass");
    }

    #[test]
    fn test_login_command_order_independence() {
        let command = CommandBuilder::new("A0001")
            .login()
            .password("pass")
            .username("user");

        assert_eq!(command.as_string(), "A0001 LOGIN user pass");
    }
}
