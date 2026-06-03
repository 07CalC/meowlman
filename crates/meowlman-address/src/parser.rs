use crate::{
    Mailbox,
    ast::{AddrSpec, Domain, LocalPart, LocalPartPart},
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
    pub fn parse(&mut self) -> Result<(), String> {
        // Implement the parsing logic here
        Ok(())
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

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let token = self.bump();
        if &token == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at pos: {:?}",
                expected, token, self.pos
            ))
        }
    }

    fn parse_display_name(&mut self) -> Result<String, String> {
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
                token => return Err(format!("unexpected token in dispaly name: {:?}", token)),
            }
        }
        Ok(parts.join(" "))
    }
    fn parse_local_part(&mut self) -> Result<LocalPart, String> {
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
            Err("Expected local part".to_string())
        } else {
            Ok(LocalPart { parts: segments })
        }
    }

    fn parse_domain(&mut self) -> Result<Domain, String> {
        let mut labels = Vec::new();
        let first = match self.bump() {
            Token::Atom(a) => a,
            token => return Err(format!("expected domain laber got: {:?}", token)),
        };
        labels.push(first);
        while matches!(self.peek(), Token::Dot) {
            self.bump();
            match self.bump() {
                Token::Atom(a) => labels.push(a),
                token => return Err(format!("expected domain label after dot, got: {:?}", token)),
            }
        }
        Ok(Domain { labels })
    }

    pub fn parse_mailbox(&mut self) -> Result<Mailbox, String> {
        let display_name = if matches!(self.peek(), Token::LessThan) {
            None
        } else {
            Some(self.parse_display_name()?)
        };

        self.expect(&Token::LessThan)?;
        let addr_spec = self.parse_addr_spec()?;
        self.expect(&Token::GreaterThan)?;
        self.expect(&Token::End)?;

        Ok(Mailbox {
            display_name,
            addr_spec,
        })
    }

    pub fn parse_addr_spec(&mut self) -> Result<AddrSpec, String> {
        let local_part = self.parse_local_part()?;
        self.expect(&Token::At)?;
        let domain = self.parse_domain()?;

        Ok(AddrSpec { local_part, domain })
    }
}
