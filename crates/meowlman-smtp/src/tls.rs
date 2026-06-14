use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

pub enum SmtpStream {
    Placeholder,
    Plain(BufReader<tokio::net::TcpStream>),
    Tls(BufReader<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>),
}

impl SmtpStream {
    pub async fn read_line(&mut self, buf: &mut String) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            SmtpStream::Plain(reader) => reader.read_line(buf).await?,
            SmtpStream::Tls(reader) => reader.read_line(buf).await?,
            SmtpStream::Placeholder => unreachable!(),
        };
        Ok(())
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            SmtpStream::Plain(reader) => reader.get_mut().write_all(data).await?,
            SmtpStream::Tls(reader) => reader.get_mut().write_all(data).await?,
            SmtpStream::Placeholder => unreachable!(),
        };
        Ok(())
    }
}
