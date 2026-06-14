use std::sync::Arc;

use crate::{conn::SmtpConnection, message_handler::MessageHandler};

const DEFAULT_HELO_NAME: &str = "meowlman-smtp";
const DEFAULT_MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_MAX_CONNECTIONS: usize = 100;
const DEFAULT_READ_TIMEOUT: u64 = 60; // seconds
const DEFAULT_WRITE_TIMEOUT: u64 = 60; // seconds
pub struct SmtpServer {
    pub host: String,
    pub port: u16,
    pub helo_name: String,
    pub tls: bool,
    pub tls_cert: Option<String>,
    pub max_message_size: Option<usize>,
    pub max_connections: Option<usize>,
    /// in seconds
    pub read_timeout: Option<u64>,
    /// in seconds
    pub write_timeout: Option<u64>,
    connections: Vec<tokio::task::JoinHandle<()>>,
    message_handler: Arc<Option<Box<dyn MessageHandler>>>,
}

impl SmtpServer {
    pub fn new(host: String, port: u16) -> Self {
        SmtpServer {
            host,
            port,
            helo_name: DEFAULT_HELO_NAME.to_string(),
            tls: false,
            tls_cert: None,
            max_message_size: Some(DEFAULT_MAX_MESSAGE_SIZE),
            max_connections: Some(DEFAULT_MAX_CONNECTIONS),
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_WRITE_TIMEOUT),
            message_handler: Arc::new(None),
            connections: Vec::new(),
        }
    }

    pub fn with_message_handler(mut self, handler: Box<dyn MessageHandler>) -> Self {
        self.message_handler = Arc::new(Some(handler));
        self
    }

    pub async fn serve(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(self.address()).await?;
        println!("SMTP server listening on {}", self.address());
        loop {
            let (stream, _) = listener.accept().await.unwrap_or_else(|e| {
                eprintln!("Failed to accept connection: {}", e);
                std::process::exit(1);
            });
            let mut connection = SmtpConnection::new(
                stream,
                self.helo_name.clone(),
                self.tls,
                self.tls_cert.clone(),
                self.max_message_size.unwrap_or(DEFAULT_MAX_MESSAGE_SIZE),
                self.read_timeout.unwrap_or(DEFAULT_READ_TIMEOUT),
                self.write_timeout.unwrap_or(DEFAULT_WRITE_TIMEOUT),
                self.message_handler.clone(),
            );
            connection.handle().await;
        }
    }

    pub fn with_helo_name(mut self, helo_name: String) -> Self {
        self.helo_name = helo_name;
        self
    }

    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = Some(size);
        self
    }

    pub fn with_max_connections(mut self, connections: usize) -> Self {
        self.max_connections = Some(connections);
        self
    }

    pub fn with_read_timeout(mut self, timeout: u64) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    pub fn with_write_timeout(mut self, timeout: u64) -> Self {
        self.write_timeout = Some(timeout);
        self
    }
    pub fn with_tls(mut self, tls: bool, tls_cert: String) -> Self {
        self.tls = tls;
        self.tls_cert = Some(tls_cert);
        self
    }
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
