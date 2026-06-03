#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid character: {0}")]
    InvalidCharacter(char),
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("invalid local part: {0}")]
    InvalidLocalPart(String),
    #[error("invalid domain: {0}")]
    InvalidDomain(String),
    #[error("invalid display name: {0}")]
    InvalidDisplayName(String),
    #[error("unterminated quoted string")]
    UnterminatedQuotedString,
    #[error("unterminated comment")]
    UnterminatedComment,
    #[error("expected `{expected}`, found `{found}`")]
    ExpectedToken { expected: String, found: String },
}
