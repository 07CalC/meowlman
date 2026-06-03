use crate::{
    ast::{AddrSpec, LocalPartPart},
    parser::Parser,
};

mod ast;
mod error;
mod lexer;
mod parser;

pub use error::ParseError;

#[derive(Debug)]
pub struct Mailbox {
    pub display_name: Option<String>,
    pub addr_spec: AddrSpec,
}

impl Mailbox {
    pub fn new(address: &str) -> Result<Mailbox, ParseError> {
        let mut lexer = lexer::Lexer::new(address);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        Ok(parser.parse_mailbox()?)
    }
}

impl AddrSpec {
    pub fn address(&self) -> String {
        let mut out = String::new();
        for (i, part) in self.local_part.parts.iter().enumerate() {
            if i > 0 {
                out.push('.');
            }
            match part {
                LocalPartPart::Atom(a) => out += a,
                LocalPartPart::QuotedString(qs) => {
                    out.push('"');
                    out += &qs;
                    out.push('"');
                }
            }
        }
        out.push('@');
        out += &self.domain.labels.join(".");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_addr_spec() {
        let m = Mailbox::new("user@example.com").unwrap();
        assert!(m.display_name.is_none());
        assert_eq!(m.addr_spec.address(), "user@example.com");
    }

    #[test]
    fn dotted_local_part() {
        let m = Mailbox::new("john.doe@example.com").unwrap();
        assert_eq!(m.addr_spec.local_part.parts.len(), 2);
        assert_eq!(m.addr_spec.address(), "john.doe@example.com");
    }

    #[test]
    fn quoted_local_part() {
        let m = Mailbox::new(r#""john.doe"@example.com"#).unwrap();
        assert_eq!(m.addr_spec.address(), r#""john.doe"@example.com"#);
    }

    #[test]
    fn display_name_and_angle() {
        let m = Mailbox::new("John Doe <john@example.com>").unwrap();
        assert_eq!(m.display_name.unwrap(), "John Doe");
        assert_eq!(m.addr_spec.address(), "john@example.com");
    }

    #[test]
    fn quoted_display_name() {
        let m = Mailbox::new(r#""John Doe" <john@example.com>"#).unwrap();
        assert_eq!(m.display_name.unwrap(), "John Doe");
        assert_eq!(m.addr_spec.address(), "john@example.com");
    }

    #[test]
    fn angle_only() {
        let m = Mailbox::new("<user@example.com>").unwrap();
        assert!(m.display_name.is_none());
        assert_eq!(m.addr_spec.address(), "user@example.com");
    }

    #[test]
    fn domain_literal_ipv4() {
        let m = Mailbox::new("user@[192.168.1.1]").unwrap();
        assert_eq!(m.addr_spec.domain.labels, vec!["[192.168.1.1]"]);
        assert_eq!(m.addr_spec.address(), "user@[192.168.1.1]");
    }

    #[test]
    fn domain_literal_ipv6() {
        let m = Mailbox::new("user@[IPv6:2001:db8::1]").unwrap();
        assert_eq!(m.addr_spec.domain.labels, vec!["[IPv6:2001:db8::1]"]);
    }

    #[test]
    fn display_name_with_domain_literal() {
        let m = Mailbox::new("Admin <admin@[10.0.0.1]>").unwrap();
        assert_eq!(m.display_name.unwrap(), "Admin");
        assert_eq!(m.addr_spec.address(), "admin@[10.0.0.1]");
    }

    #[test]
    fn empty_input() {
        let err = Mailbox::new("").unwrap_err();
        assert!(matches!(err, ParseError::InvalidDisplayName(_)));
    }

    #[test]
    fn missing_at() {
        let err = Mailbox::new("notanemail").unwrap_err();
        assert!(matches!(err, ParseError::ExpectedToken { .. }));
    }

    #[test]
    fn missing_domain() {
        let err = Mailbox::new("user@").unwrap_err();
        assert!(matches!(err, ParseError::InvalidDomain(_)));
    }

    #[test]
    fn comments_between_tokens() {
        let m = Mailbox::new("user(comment)@example.com").unwrap();
        assert_eq!(m.addr_spec.address(), "user@example.com");
    }

    #[test]
    fn nested_comments() {
        let m = Mailbox::new("user@(nested(deep)comment)example.com").unwrap();
        assert_eq!(m.addr_spec.address(), "user@example.com");
    }

    #[test]
    fn comment_in_display_name() {
        let m = Mailbox::new("John(nickname)Doe <john@example.com>").unwrap();
        assert_eq!(m.display_name.unwrap(), "John Doe");
        assert_eq!(m.addr_spec.address(), "john@example.com");
    }

    #[test]
    fn display_name_with_special_chars() {
        let m = Mailbox::new("hello!world <a@b.com>").unwrap();
        assert_eq!(m.display_name.unwrap(), "hello!world");
    }

    #[test]
    fn multiple_dots_in_local_part() {
        let m = Mailbox::new("a.b.c.d@example.com").unwrap();
        assert_eq!(m.addr_spec.local_part.parts.len(), 4);
        assert_eq!(m.addr_spec.address(), "a.b.c.d@example.com");
    }

    #[test]
    fn subdomain_domain() {
        let m = Mailbox::new("user@sub.example.com").unwrap();
        assert_eq!(m.addr_spec.domain.labels, vec!["sub", "example", "com"]);
    }

    #[test]
    fn quoted_string_with_escaped_char() {
        let m = Mailbox::new(r#""john\ doe"@example.com"#).unwrap();
        assert_eq!(m.addr_spec.address(), r#""john doe"@example.com"#);
    }

    #[test]
    fn unterminated_quoted_string() {
        let err = Mailbox::new(r#""unclosed@example.com"#).unwrap_err();
        assert!(matches!(err, ParseError::UnterminatedQuotedString));
    }

    #[test]
    fn unterminated_comment() {
        let err = Mailbox::new("user(a@example.com").unwrap_err();
        assert!(matches!(err, ParseError::UnterminatedComment));
    }
}
