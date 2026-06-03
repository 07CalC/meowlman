#[derive(Debug, thiserror::Error)]
pub enum SmtpClientError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unexpected smtp response: expected {expected}, got {code}: {detail}")]
    UnexpectedResponse {
        expected: String,
        code: u16,
        detail: String,
    },
    #[error("invalid smtp response: {0}")]
    InvalidResponse(String),
    #[error("inconsistent multi-line response codes: {first} and {second}")]
    InconsistentCodes { first: u16, second: u16 },
    #[error("server closed connection unexpectedly")]
    ConnectionClosed,
}
