use crate::{
    Mailbox,
    ast::{AddrSpec, Domain, LocalPart, LocalPartPart},
    error::ParseError,
    lexer::Token,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::End)
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if !matches!(token, Token::End) {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let token = self.bump();
        if &token == expected {
            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
            })
        }
    }

    /// Check if a `<` token appears before any `@` or end-of-input.
    /// Used to disambiguate display-name + angle-addr vs bare addr-spec.
    fn has_angle_before_at(&self) -> bool {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::LessThan => return true,
                Token::At => return false,
                _ => i += 1,
            }
        }
        false
    }

    fn parse_display_name(&mut self) -> Result<String, ParseError> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Token::Atom(atom) => {
                    parts.push(atom.clone());
                    self.bump();
                }
                Token::QuotedString(qs) => {
                    parts.push(qs.clone());
                    self.bump();
                }
                Token::LessThan => break,
                token => {
                    return Err(ParseError::InvalidDisplayName(format!(
                        "unexpected token in display name: {:?}",
                        token
                    )));
                }
            }
        }
        if parts.is_empty() {
            return Err(ParseError::InvalidDisplayName(
                "empty display name".into(),
            ));
        }
        Ok(parts.join(" "))
    }

    fn parse_local_part(&mut self) -> Result<LocalPart, ParseError> {
        let mut segments = Vec::new();
        loop {
            match self.peek() {
                Token::Atom(atom) => {
                    segments.push(LocalPartPart::Atom(atom.clone()));
                    self.bump();
                }
                Token::QuotedString(qs) => {
                    segments.push(LocalPartPart::QuotedString(qs.clone()));
                    self.bump();
                }
                Token::Dot => {
                    self.bump();
                }
                _ => break,
            }
        }
        if segments.is_empty() {
            Err(ParseError::InvalidLocalPart("expected local part".into()))
        } else {
            Ok(LocalPart { parts: segments })
        }
    }

    fn parse_domain(&mut self) -> Result<Domain, ParseError> {
        // Domain literal: [1.2.3.4] or [IPv6:...]
        if matches!(self.peek(), Token::LBracket) {
            return self.parse_domain_literal();
        }

        let mut labels = Vec::new();
        let first = match self.bump() {
            Token::Atom(a) => a,
            token => {
                return Err(ParseError::InvalidDomain(format!(
                    "expected domain label, got: {:?}",
                    token
                )));
            }
        };
        labels.push(first);
        while matches!(self.peek(), Token::Dot) {
            self.bump();
            match self.bump() {
                Token::Atom(a) => labels.push(a),
                token => {
                    return Err(ParseError::InvalidDomain(format!(
                        "expected domain label after dot, got: {:?}",
                        token
                    )));
                }
            }
        }
        Ok(Domain { labels })
    }

    fn parse_domain_literal(&mut self) -> Result<Domain, ParseError> {
        self.expect(&Token::LBracket)?;
        let mut content = String::new();
        loop {
            match self.peek() {
                Token::RBracket => {
                    self.bump();
                    break;
                }
                Token::End => {
                    return Err(ParseError::InvalidDomain(
                        "unterminated domain literal".into(),
                    ));
                }
                Token::Atom(a) => {
                    content.push_str(a);
                    self.bump();
                }
                Token::Dot => {
                    content.push('.');
                    self.bump();
                }
                Token::Colon => {
                    content.push(':');
                    self.bump();
                }
                other => {
                    return Err(ParseError::InvalidDomain(format!(
                        "unexpected token in domain literal: {:?}",
                        other
                    )));
                }
            }
        }
        Ok(Domain {
            labels: vec![format!("[{}]", content)],
        })
    }

    pub fn parse_mailbox(&mut self) -> Result<Mailbox, ParseError> {
        if matches!(self.peek(), Token::LessThan) {
            // `<addr-spec>` — no display name
            self.expect(&Token::LessThan)?;
            let addr_spec = self.parse_addr_spec()?;
            self.expect(&Token::GreaterThan)?;
            self.expect(&Token::End)?;
            return Ok(Mailbox {
                display_name: None,
                addr_spec,
            });
        }

        if matches!(self.peek(), Token::End) {
            return Err(ParseError::InvalidDisplayName(
                "empty input".into(),
            ));
        }

        // Could be "display-name <addr-spec>" or bare "addr-spec"
        if self.has_angle_before_at() {
            // display-name <addr-spec>
            let display_name = Some(self.parse_display_name()?);
            self.expect(&Token::LessThan)?;
            let addr_spec = self.parse_addr_spec()?;
            self.expect(&Token::GreaterThan)?;
            self.expect(&Token::End)?;
            Ok(Mailbox {
                display_name,
                addr_spec,
            })
        } else {
            // bare addr-spec: user@domain
            let addr_spec = self.parse_addr_spec()?;
            self.expect(&Token::End)?;
            Ok(Mailbox {
                display_name: None,
                addr_spec,
            })
        }
    }

    pub fn parse_addr_spec(&mut self) -> Result<AddrSpec, ParseError> {
        let local_part = self.parse_local_part()?;
        self.expect(&Token::At)?;
        let domain = self.parse_domain()?;
        Ok(AddrSpec { local_part, domain })
    }
}
