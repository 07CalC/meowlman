use std::str::FromStr;

use crate::parser::Parser;

mod ast;
mod error;
mod lexer;
mod parser;

pub use ast::{AddrSpec, Domain, LocalPart, LocalPartPart};
pub use error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub struct Mailbox {
    pub display_name: Option<String>,
    pub local_part: String,
    pub domain: String,
}

impl Mailbox {
    pub fn new(display_name: Option<String>, local_part: &str, domain: &str) -> Self {
        Self {
            display_name,
            local_part: local_part.to_string(),
            domain: domain.to_string(),
        }
    }

    /// Parses an email address from a string.
    /// The input can be in the form of `local_part@domain` or `Display Name <local_part@domain>`.
    pub fn from_str(address: &str) -> Result<Self, ParseError> {
        let mut lexer = lexer::Lexer::new(address);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_mailbox()
    }

    /// returns the email address in the form `local_part@domain`.
    /// this does not include the display name or angle brackets.
    /// for example, for `John Doe <john.doe@mail.com>`
    /// this will return `john.doe@mail.com`.
    pub fn address(&self) -> String {
        format!("{}@{}", self.local_part, self.domain)
    }
}

impl FromStr for Mailbox {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Mailbox::from_str(s)
    }
}

impl std::fmt::Display for Mailbox {
    /// formats the mailbox as `Display Name <local_part@domain>` if a display name is present,
    /// otherwise just `local_part@domain`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref name) = self.display_name {
            write!(f, "{} <{}>", name, self.address())
        } else {
            write!(f, "{}", self.address())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_access() {
        let m = Mailbox::new(Some("Me".into()), "user", "example.com");
        assert_eq!(m.display_name.as_deref(), Some("Me"));
        assert_eq!(m.local_part, "user");
        assert_eq!(m.domain, "example.com");
    }

    #[test]
    fn field_access_after_parse() {
        let m = Mailbox::from_str("A B <a.b@c.d>").unwrap();
        assert_eq!(m.display_name.as_deref(), Some("A B"));
        assert_eq!(m.local_part, "a.b");
        assert_eq!(m.domain, "c.d");
    }

    #[test]
    fn no_display_name() {
        let m = Mailbox::new(None, "x", "y.z");
        assert!(m.display_name.is_none());
        assert_eq!(m.local_part, "x");
        assert_eq!(m.domain, "y.z");
    }

    #[test]
    fn construct_simple() {
        let m = Mailbox::new(None, "user", "example.com");
        assert_eq!(m.address(), "user@example.com");
        assert_eq!(m.to_string(), "user@example.com");
    }

    #[test]
    fn construct_with_display_name() {
        let m = Mailbox::new(Some("John Doe".into()), "john.doe", "example.com");
        assert_eq!(m.display_name.as_deref().unwrap(), "John Doe");
        assert_eq!(m.to_string(), "John Doe <john.doe@example.com>");
    }

    #[test]
    fn construct_dotted_local() {
        let m = Mailbox::new(None, "a.b.c", "d.com");
        assert_eq!(m.local_part, "a.b.c");
        assert_eq!(m.to_string(), "a.b.c@d.com");
    }

    #[test]
    fn construct_subdomain() {
        let m = Mailbox::new(None, "user", "sub.example.com");
        assert_eq!(m.domain, "sub.example.com");
    }

    #[test]
    fn parse_bare_addr_spec() {
        let m = Mailbox::from_str("user@example.com").unwrap();
        assert!(m.display_name.is_none());
        assert_eq!(m.address(), "user@example.com");
    }

    #[test]
    fn parse_dotted_local_part() {
        let m = Mailbox::from_str("john.doe@example.com").unwrap();
        assert_eq!(m.local_part, "john.doe");
        assert_eq!(m.address(), "john.doe@example.com");
    }

    #[test]
    fn parse_quoted_local_part() {
        let m = Mailbox::from_str(r#""john.doe"@example.com"#).unwrap();
        assert_eq!(m.local_part, r#""john.doe""#);
        assert_eq!(m.address(), r#""john.doe"@example.com"#);
    }

    #[test]
    fn parse_display_name_and_angle() {
        let m = Mailbox::from_str("John Doe <john@example.com>").unwrap();
        assert_eq!(m.display_name.as_deref().unwrap(), "John Doe");
        assert_eq!(m.address(), "john@example.com");
    }

    #[test]
    fn parse_quoted_display_name() {
        let m = Mailbox::from_str(r#""John Doe" <john@example.com>"#).unwrap();
        assert_eq!(m.display_name.as_deref().unwrap(), "John Doe");
        assert_eq!(m.address(), "john@example.com");
    }

    #[test]
    fn parse_angle_only() {
        let m = Mailbox::from_str("<user@example.com>").unwrap();
        assert!(m.display_name.is_none());
        assert_eq!(m.address(), "user@example.com");
    }

    #[test]
    fn parse_domain_literal_ipv4() {
        let m = Mailbox::from_str("user@[192.168.1.1]").unwrap();
        assert_eq!(m.domain, "[192.168.1.1]");
        assert_eq!(m.address(), "user@[192.168.1.1]");
    }

    #[test]
    fn parse_domain_literal_ipv6() {
        let m = Mailbox::from_str("user@[IPv6:2001:db8::1]").unwrap();
        assert_eq!(m.domain, "[IPv6:2001:db8::1]");
    }

    #[test]
    fn parse_display_name_with_domain_literal() {
        let m = Mailbox::from_str("Admin <admin@[10.0.0.1]>").unwrap();
        assert_eq!(m.display_name.as_deref().unwrap(), "Admin");
        assert_eq!(m.address(), "admin@[10.0.0.1]");
    }

    #[test]
    fn parse_empty_is_error() {
        let err = Mailbox::from_str("").unwrap_err();
        assert!(matches!(err, ParseError::InvalidDisplayName(_)));
    }

    #[test]
    fn parse_missing_at() {
        let err = Mailbox::from_str("notanemail").unwrap_err();
        assert!(matches!(err, ParseError::ExpectedToken { .. }));
    }

    #[test]
    fn parse_missing_domain() {
        let err = Mailbox::from_str("user@").unwrap_err();
        assert!(matches!(err, ParseError::InvalidDomain(_)));
    }

    #[test]
    fn parse_comments_between_tokens() {
        let m = Mailbox::from_str("user(comment)@example.com").unwrap();
        assert_eq!(m.address(), "user@example.com");
    }

    #[test]
    fn parse_nested_comments() {
        let m = Mailbox::from_str("user@(nested(deep)comment)example.com").unwrap();
        assert_eq!(m.address(), "user@example.com");
    }

    #[test]
    fn parse_comment_in_display_name() {
        let m = Mailbox::from_str("John(nickname)Doe <john@example.com>").unwrap();
        assert_eq!(m.display_name.as_deref().unwrap(), "John Doe");
        assert_eq!(m.address(), "john@example.com");
    }

    #[test]
    fn parse_display_name_with_special_chars() {
        let m = Mailbox::from_str("hello!world <a@b.com>").unwrap();
        assert_eq!(m.display_name.as_deref().unwrap(), "hello!world");
    }

    #[test]
    fn parse_multiple_dots_in_local_part() {
        let m = Mailbox::from_str("a.b.c.d@example.com").unwrap();
        assert_eq!(m.local_part, "a.b.c.d");
    }

    #[test]
    fn parse_subdomain_domain() {
        let m = Mailbox::from_str("user@sub.example.com").unwrap();
        assert_eq!(m.domain, "sub.example.com");
    }

    #[test]
    fn parse_quoted_string_with_escaped_char() {
        let m = Mailbox::from_str(r#""john\ doe"@example.com"#).unwrap();
        assert_eq!(m.local_part, r#""john doe""#);
        assert_eq!(m.address(), r#""john doe"@example.com"#);
    }

    #[test]
    fn parse_unterminated_quoted_string() {
        let err = Mailbox::from_str(r#""unclosed@example.com"#).unwrap_err();
        assert!(matches!(err, ParseError::UnterminatedQuotedString));
    }

    #[test]
    fn parse_unterminated_comment() {
        let err = Mailbox::from_str("user(a@example.com").unwrap_err();
        assert!(matches!(err, ParseError::UnterminatedComment));
    }

    #[test]
    fn from_str_trait() {
        let m: Mailbox = "a@b.com".parse().unwrap();
        assert_eq!(m.to_string(), "a@b.com");
    }

    #[test]
    fn construct_and_format() {
        let m = Mailbox::new(Some("Alice".into()), "alice", "example.com");
        assert_eq!(m.to_string(), "Alice <alice@example.com>");
    }

    #[test]
    fn parse_and_format() {
        let m = Mailbox::from_str("Bob <bob@example.com>").unwrap();
        assert_eq!(m.to_string(), "Bob <bob@example.com>");
    }
}
