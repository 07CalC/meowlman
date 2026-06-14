use std::{
    fs::File,
    io::BufReader,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use tokio_rustls::{
    TlsAcceptor,
    rustls::{self, PrivateKey},
};

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
    pub tls_acceptor: Option<Arc<TlsAcceptor>>,
    pub max_message_size: Option<usize>,
    pub max_connections: Option<usize>,
    /// in seconds
    pub read_timeout: Option<u64>,
    /// in seconds
    pub write_timeout: Option<u64>,
    message_handler: Arc<Option<Box<dyn MessageHandler>>>,
    connections: Arc<AtomicUsize>,
}

impl SmtpServer {
    pub fn new(host: String, port: u16) -> Self {
        SmtpServer {
            host,
            port,
            helo_name: DEFAULT_HELO_NAME.to_string(),
            tls: false,
            tls_acceptor: None,
            max_message_size: Some(DEFAULT_MAX_MESSAGE_SIZE),
            max_connections: Some(DEFAULT_MAX_CONNECTIONS),
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_WRITE_TIMEOUT),
            message_handler: Arc::new(None),
            connections: Arc::new(AtomicUsize::new(0)),
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
            let hello_name = self.helo_name.clone();
            let max_message_size = self
                .max_message_size
                .clone()
                .unwrap_or(DEFAULT_MAX_MESSAGE_SIZE);
            let read_timeout = self.read_timeout.clone().unwrap_or(DEFAULT_READ_TIMEOUT);
            let write_timeout = self.write_timeout.clone().unwrap_or(DEFAULT_WRITE_TIMEOUT);
            let message_handler = self.message_handler.clone();
            let connections = self.connections.clone();
            let tls_acceptor = self.tls_acceptor.clone();
            tokio::spawn(async move {
                connections.fetch_add(1, Ordering::Relaxed);
                let mut connection = SmtpConnection::new(
                    stream,
                    hello_name,
                    tls_acceptor,
                    max_message_size,
                    read_timeout,
                    write_timeout,
                    message_handler,
                );
                if let Err(e) = connection.handle().await {
                    eprintln!("Connection error: {}", e);
                }
                connections.fetch_sub(1, Ordering::Relaxed);
            });
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
    pub fn with_start_tls(
        mut self,
        tls_cert_path: &str,
        tls_key_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.tls = true;

        let cert_file = File::open(&tls_cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut cert_reader)
            .map_err(|_| "Failed to read TLS certificate".to_string())?
            .into_iter()
            .map(rustls::Certificate)
            .collect();
        let key_file = File::open(&tls_key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
            .map_err(|_| "Failed to read TLS private key".to_string())?;
        if keys.is_empty() {
            return Err("No TLS private keys found".into());
        }
        let key = PrivateKey(keys[0].clone());
        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("Failed to create TLS config: {}", e))?;
        self.tls_acceptor = Some(Arc::new(TlsAcceptor::from(Arc::new(config))));
        Ok(self)
    }
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
