use crate::error::ParseError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Atom(String),
    QuotedString(String),
    At,          // @
    Dot,         // .
    Comma,       // ,
    Colon,       // :
    Semicolon,   // ;
    LessThan,    // <
    GreaterThan, // >
    LBracket,    // [
    RBracket,    // ]
    End,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while !self.eof() {
            let token = self.next_token()?;
            if let Token::End = token {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_atom(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if Self::is_atext(ch) {
                self.bump();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn skip_wsp(&mut self) -> Result<(), ParseError> {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else if ch == '(' {
                // comments are consumed and discarded during whitespace skipping
                // so they don't appear as tokens in the token stream
                self.eat_comment()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_wsp()?;
        let Some(ch) = self.peek() else {
            return Ok(Token::End);
        };
        let token = match ch {
            '@' => Token::At,
            '.' => Token::Dot,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '"' => {
                let quoted = self.consume_quoted_string()?;
                return Ok(Token::QuotedString(quoted));
            }
            _ if Self::is_atext(ch) => {
                let atom = self.consume_atom();
                return Ok(Token::Atom(atom));
            }
            _ => return Err(ParseError::InvalidCharacter(ch)),
        };
        self.bump();
        Ok(token)
    }

    fn consume_quoted_string(&mut self) -> Result<String, ParseError> {
        let quote = self.bump();
        if quote != Some('"') {
            return Err(ParseError::InvalidCharacter('"'));
        }
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(ParseError::UnterminatedQuotedString),
                Some('"') => {
                    self.bump();
                    return Ok(out);
                }
                Some('\\') => {
                    self.bump(); // skip backslash
                    match self.bump() {
                        Some(c) => out.push(c),
                        None => return Err(ParseError::UnterminatedQuotedString),
                    }
                }
                Some(ch) => {
                    out.push(ch);
                    self.bump();
                }
            }
        }
    }

    fn eat_comment(&mut self) -> Result<String, ParseError> {
        if self.bump() != Some('(') {
            return Err(ParseError::InvalidCharacter('('));
        }
        let mut out = String::new();
        let mut depth = 1;
        while let Some(ch) = self.bump() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(out);
                    }
                }
                '\\' => {
                    self.bump().ok_or(ParseError::UnterminatedComment)?;
                }
                _ => out.push(ch),
            }
        }
        Err(ParseError::UnterminatedComment)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5322#section-3.2.3
    ///
    ///   atext        =   ALPHA / DIGIT /    ; Printable US-ASCII
    ///                "!" / "#" /        ;  characters not including
    ///                "$" / "%" /        ;  specials.  Used for atoms.
    ///                "&" / "'" /
    ///                "*" / "+" /
    ///                "-" / "/" /
    ///                "=" / "?" /
    ///                "^" / "_" /
    ///                "`" / "{" /
    ///                "|" / "}" /
    ///                "~"
    fn is_atext(ch: char) -> bool {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '!' | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '-'
                    | '/'
                    | '='
                    | '?'
                    | '^'
                    | '_'
                    | '`'
                    | '{'
                    | '|'
                    | '}'
                    | '~'
            )
    }
}
