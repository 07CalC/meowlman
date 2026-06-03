#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("invalid local part")]
    InvalidLocalPart,
    #[error("invalid domain: {0}")]
    InvalidDomain(String),
}
