#[derive(Debug, Clone, PartialEq)]
pub enum Flag {
    System(SystemFlag),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemFlag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
}

/// Mailbox status information
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxStatus {
    pub messages: u32,
    pub recent: u32,
    pub uid_next: u32,
    pub uid_validity: u32,
    pub unseen: Option<u32>,
    pub flags: Vec<Flag>,
    pub permanent_flags: Vec<Flag>,
}

/// Sequence set for IMAP commands
#[derive(Debug, Clone, PartialEq)]
pub enum SequenceSet {
    Single(u32),
    Range(u32, Option<u32>), // None means "*"
    List(Vec<SequenceSet>),
}

/// Mailbox information
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxInfo {
    pub name: String,
    pub flags: Vec<Flag>,
    pub permanent_flags: Vec<Flag>,
    pub uid_validity: u32,
    pub uid_next: u32,
    pub exists: u32,
    pub recent: u32,
}

// Simple parsing functions (without nom for now)

/// Parse a mailbox name (INBOX, folder names, etc.)
pub fn parse_mailbox_name(input: &str) -> Result<String, String> {
    if input.is_empty() {
        return Err("Empty mailbox name".to_string());
    }

    let name = input.split_whitespace().next().unwrap_or("");
    Ok(name.to_string())
}

/// Parse IMAP flags like (\Seen \Flagged $Custom)
pub fn parse_flags(input: &str) -> Result<Vec<Flag>, String> {
    let input = input.trim();
    if !input.starts_with('(') || !input.ends_with(')') {
        return Err("Flags must be enclosed in parentheses".to_string());
    }

    let inner = &input[1..input.len() - 1].trim();
    if inner.is_empty() {
        return Ok(vec![]);
    }

    let flag_strs: Vec<&str> = inner.split_whitespace().collect();
    let mut flags = Vec::new();

    for flag_str in flag_strs {
        let flag = match flag_str {
            "\\Seen" => Flag::System(SystemFlag::Seen),
            "\\Answered" => Flag::System(SystemFlag::Answered),
            "\\Flagged" => Flag::System(SystemFlag::Flagged),
            "\\Deleted" => Flag::System(SystemFlag::Deleted),
            "\\Draft" => Flag::System(SystemFlag::Draft),
            "\\Recent" => Flag::System(SystemFlag::Recent),
            s => Flag::Keyword(s.to_string()),
        };
        flags.push(flag);
    }

    Ok(flags)
}

/// Parse sequence sets like "1:*", "1,3", "1:5"
pub fn parse_sequence_set(input: &str) -> Result<SequenceSet, String> {
    if input.contains(',') {
        // Multiple sequences: "1,3,5" or "1:3,5,7:*"
        let parts: Result<Vec<_>, _> = input.split(',').map(parse_single_sequence).collect();
        Ok(SequenceSet::List(parts?))
    } else {
        parse_single_sequence(input)
    }
}

fn parse_single_sequence(input: &str) -> Result<SequenceSet, String> {
    if input.contains(':') {
        // Range: "1:5" or "1:*"
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format: {}", input));
        }

        let start = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid start number: {}", parts[0]))?;

        let end = if parts[1] == "*" {
            None
        } else {
            Some(
                parts[1]
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid end number: {}", parts[1]))?,
            )
        };

        Ok(SequenceSet::Range(start, end))
    } else if input == "*" {
        // Special case: just "*"
        Ok(SequenceSet::Range(u32::MAX, None))
    } else {
        // Single number: "5"
        let num = input
            .parse::<u32>()
            .map_err(|_| format!("Invalid number: {}", input))?;
        Ok(SequenceSet::Single(num))
    }
}

/// Check if sequence set matches a given message number
pub fn sequence_matches(seq: &SequenceSet, msg_num: u32, max_msg: u32) -> bool {
    match seq {
        SequenceSet::Single(n) => {
            if *n == u32::MAX {
                // "*" case
                msg_num == max_msg
            } else {
                msg_num == *n
            }
        }
        SequenceSet::Range(start, end) => {
            let actual_start = if *start == u32::MAX { max_msg } else { *start };
            let actual_end = end.unwrap_or(max_msg);
            msg_num >= actual_start && msg_num <= actual_end
        }
        SequenceSet::List(sequences) => sequences
            .iter()
            .any(|s| sequence_matches(s, msg_num, max_msg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MSG: &str = r#"To: test@test
From: test <test@test>
Subject: test
Date: Tue, 8 May 2018 20:48:21 +0000
Content-Type: text/plain; charset=utf-8
Content-Transfer-Encoding: 7bit
Cc: foo <foo@foo>, bar <bar@bar>
X-CustomHeader: foo

Test! Test! Test! Test!
"#;

    #[test]
    fn test_parse_mailbox_name() {
        let input = "INBOX";
        let result = parse_mailbox_name(input);
        assert!(result.is_ok());
        let name = result.unwrap();
        assert_eq!(name, "INBOX");

        let input = "Sent Items extra";
        let result = parse_mailbox_name(input);
        assert!(result.is_ok());
        let name = result.unwrap();
        assert_eq!(name, "Sent");
    }

    #[test]
    fn test_parse_flags() {
        // Empty flags
        let input = "()";
        let result = parse_flags(input);
        assert!(result.is_ok());
        let flags = result.unwrap();
        assert_eq!(flags, vec![]);

        // Single flag
        let input = "(\\Seen)";
        let result = parse_flags(input);
        assert!(result.is_ok());
        let flags = result.unwrap();
        assert_eq!(flags, vec![Flag::System(SystemFlag::Seen)]);

        // Multiple flags (like from Go tests)
        let input = "(\\Seen \\Flagged $Custom)";
        let result = parse_flags(input);
        assert!(result.is_ok());
        let flags = result.unwrap();
        assert_eq!(
            flags,
            vec![
                Flag::System(SystemFlag::Seen),
                Flag::System(SystemFlag::Flagged),
                Flag::Keyword("$Custom".to_string())
            ]
        );

        // Test flags from Go test cases
        let input = "($Test1 $Test2 \\Recent)";
        let result = parse_flags(input);
        assert!(result.is_ok());
        let flags = result.unwrap();
        assert_eq!(
            flags,
            vec![
                Flag::Keyword("$Test1".to_string()),
                Flag::Keyword("$Test2".to_string()),
                Flag::System(SystemFlag::Recent)
            ]
        );
    }

    #[test]
    fn test_parse_sequence_set() {
        // Single numbers
        let result = parse_sequence_set("1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Single(1));

        let result = parse_sequence_set("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Single(42));

        // Star (*)
        let result = parse_sequence_set("*");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Range(u32::MAX, None));

        // Ranges
        let result = parse_sequence_set("1:5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Range(1, Some(5)));

        let result = parse_sequence_set("1:*");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Range(1, None));

        let result = parse_sequence_set("10:20");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SequenceSet::Range(10, Some(20)));

        // Lists (from Go test cases)
        let result = parse_sequence_set("1,3");
        assert!(result.is_ok());
        match result.unwrap() {
            SequenceSet::List(list) => {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], SequenceSet::Single(1));
                assert_eq!(list[1], SequenceSet::Single(3));
            }
            _ => panic!("Expected List variant"),
        }

        let result = parse_sequence_set("1,3,5");
        assert!(result.is_ok());
        match result.unwrap() {
            SequenceSet::List(list) => {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0], SequenceSet::Single(1));
                assert_eq!(list[1], SequenceSet::Single(3));
                assert_eq!(list[2], SequenceSet::Single(5));
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_sequence_matches() {
        // Test cases from Go test suite
        let test_cases = vec![
            // (sequence_set_str, msg_num, max_msg, should_match)
            ("1", 1, 3, true),
            ("1", 2, 3, false),
            ("1", 3, 3, false),
            ("*", 1, 3, false),
            ("*", 2, 3, false),
            ("*", 3, 3, true),
            ("1:3", 1, 3, true),
            ("1:3", 2, 3, true),
            ("1:3", 3, 3, true),
            ("1:*", 1, 3, true),
            ("1:*", 2, 3, true),
            ("1:*", 3, 3, true),
            ("2:3", 1, 3, false),
            ("2:3", 2, 3, true),
            ("2:3", 3, 3, true),
            ("1,3", 1, 3, true),
            ("1,3", 2, 3, false),
            ("1,3", 3, 3, true),
        ];

        for (seq_str, msg_num, max_msg, expected) in test_cases {
            let seq = parse_sequence_set(seq_str).unwrap();
            let matches = sequence_matches(&seq, msg_num, max_msg);
            assert_eq!(
                matches, expected,
                "Sequence '{}' with msg {} (max {}) should match: {}",
                seq_str, msg_num, max_msg, expected
            );
        }
    }

    #[test]
    fn test_mailbox_status_creation() {
        // Test creating basic mailbox status (like in Go tests)
        let status = MailboxStatus {
            messages: 2,
            recent: 2,
            uid_next: 3,
            uid_validity: 1234567890,
            unseen: Some(2),
            flags: vec![
                Flag::Keyword("$Test1".to_string()),
                Flag::Keyword("$Test2".to_string()),
                Flag::System(SystemFlag::Recent),
            ],
            permanent_flags: vec![
                Flag::Keyword("$Test1".to_string()),
                Flag::Keyword("$Test2".to_string()),
            ],
        };

        assert_eq!(status.messages, 2);
        assert_eq!(status.recent, 2);
        assert_eq!(status.uid_next, 3);
        assert_eq!(status.unseen, Some(2));
        assert_eq!(status.flags.len(), 3);
        assert_eq!(status.permanent_flags.len(), 2);
    }

    #[test]
    fn test_flag_operations() {
        // Test flag deduplication and sorting (like Go tests do)
        let mut flags = vec![
            Flag::Keyword("$Test2".to_string()),
            Flag::Keyword("$Test1".to_string()),
            Flag::System(SystemFlag::Recent),
            Flag::Keyword("$Test1".to_string()), // Duplicate
        ];

        // Remove duplicates (in real implementation)
        flags.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        flags.dedup();

        assert_eq!(flags.len(), 3); // Should have removed duplicate
    }

    #[test]
    fn test_invalid_sequence_sets() {
        // Test error cases
        assert!(parse_sequence_set("").is_err());
        assert!(parse_sequence_set("abc").is_err());
        assert!(parse_sequence_set("1:2:3").is_err());
        assert!(parse_sequence_set("1:abc").is_err());
    }

    #[test]
    fn test_edge_case_sequences() {
        // Test edge cases from Go tests
        let result = parse_sequence_set("45:30");
        assert!(result.is_ok());
        // This represents an invalid range, but parsing should succeed
        // The logic of "empty result" happens at the matching level
        match result.unwrap() {
            SequenceSet::Range(45, Some(30)) => {
                // This range makes no sense (start > end) but parses
                // sequence_matches should handle this correctly
                assert!(!sequence_matches(&SequenceSet::Range(45, Some(30)), 1, 3));
                assert!(!sequence_matches(&SequenceSet::Range(45, Some(30)), 40, 50));
            }
            _ => panic!("Expected Range variant"),
        }
    }

    #[test]
    fn test_test_message_constant() {
        // Verify our test message constant matches Go tests
        assert!(TEST_MSG.contains("To: test@test"));
        assert!(TEST_MSG.contains("From: test <test@test>"));
        assert!(TEST_MSG.contains("Subject: test"));
        assert!(TEST_MSG.contains("X-CustomHeader: foo"));
        assert!(TEST_MSG.contains("Test! Test! Test! Test!"));
        assert_eq!(TEST_MSG.len(), 232); // Should match Go constant
    }
}
