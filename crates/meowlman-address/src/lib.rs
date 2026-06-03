use crate::{
    ast::{AddrSpec, LocalPartPart},
    parser::Parser,
};

mod ast;
mod error;
mod lexer;
mod parser;

#[derive(Debug)]
pub struct Mailbox {
    pub display_name: Option<String>,
    pub addr_spec: AddrSpec,
}

impl Mailbox {
    pub fn new(address: &str) -> Result<Mailbox, String> {
        let mut lexer = lexer::Lexer::new(&address);
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
                LocalPartPart::Atom(a) => out += &a,
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
