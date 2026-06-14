use std::{sync::Arc, time::Duration};

use meowlman_address::Mailbox;
use tokio::{io::BufReader, time::timeout};

use crate::{envelope::SmtpEnvelope, message_handler::MessageHandler, tls::SmtpStream};
pub struct SmtpConnection {
    stream: SmtpStream,
    /// The HELO/EHLO name provided by the server. This is used in the initial greeting and in
    /// responses to the HELO/EHLO command.
    helo_name: String,
    tls_acceptor: Option<Arc<tokio_rustls::TlsAcceptor>>,
    max_message_size: usize,
    read_timeout: u64,
    write_timeout: u64,
    message_handler: Arc<Option<Box<dyn MessageHandler>>>,
    /// The HELO/EHLO name provided by the client, if any. This is used to determine the client's
    /// identity and capabilities.
    smtp_helo: Option<String>,
    /// The MAIL FROM address provided by the client, if any. This is used to determine the
    /// sender
    mail_from: Option<Mailbox>,
    /// The RCPT TO addresses provided by the client, if any. This is used to determine the recipients.
    rcpt_to: Vec<Mailbox>,
    /// The message data provided by the client, if any. This is used to determine the message
    /// content.
    message_data: Option<Vec<u8>>,
    /// The IP address of the client, if any. This is used to determine the client's identity and
    ip: Option<String>,
    helo_seen: bool,
    mail_seen: bool,
    rcpt_seen: bool,
}

impl SmtpConnection {
    pub fn new(
        stream: tokio::net::TcpStream,
        helo_name: String,
        tls_acceptor: Option<Arc<tokio_rustls::TlsAcceptor>>,
        max_message_size: usize,
        read_timeout: u64,
        write_timeout: u64,
        message_handler: Arc<Option<Box<dyn MessageHandler>>>,
    ) -> Self {
        SmtpConnection {
            stream: SmtpStream::Plain(BufReader::new(stream)),
            helo_name,
            max_message_size,
            read_timeout,
            write_timeout,
            message_handler,
            smtp_helo: None,
            mail_from: None,
            rcpt_to: Vec::new(),
            message_data: None,
            ip: None,
            helo_seen: false,
            mail_seen: false,
            rcpt_seen: false,
            tls_acceptor,
        }
    }

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.write_response(220, &format!("{} SMTP Service Ready", self.helo_name))
            .await?;
        self.ip = match self.stream {
            SmtpStream::Plain(ref reader) => Some(reader.get_ref().peer_addr()?.ip().to_string()),
            // TODO: extract IP from TLS stream
            SmtpStream::Tls(_) => None,
            SmtpStream::Placeholder => None,
        };
        loop {
            let line: String = self.read_line().await?;
            if line.is_empty() {
                break;
            }
            let command = line.split_whitespace().next().unwrap_or("").to_uppercase();
            match command.as_str() {
                "HELO" => self.handle_helo(&line).await?,
                "EHLO" => self.handle_ehlo(&line).await?,
                "MAIL" => self.handle_mail(&line).await?,
                "RCPT" => self.handle_rcpt(&line).await?,
                "DATA" => self.handle_data().await?,
                "RSET" => self.handle_reset().await?,
                "STARTTLS" => self.handle_starttls().await?,
                "QUIT" => {
                    self.write_response(221, "Bye").await?;
                    break;
                }
                _ => self.write_response(502, "Command not implemented").await?,
            }
        }
        Ok(())
    }

    async fn handle_starttls(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let acceptor = match &self.tls_acceptor {
            Some(a) => Arc::clone(a),
            None => {
                self.write_response(502, "Command not implemented").await?;
                return Ok(());
            }
        };

        self.write_response(220, "Ready to start TLS").await?;

        let plain_stream = match std::mem::replace(&mut self.stream, SmtpStream::Placeholder) {
            SmtpStream::Plain(reader) => reader.into_inner(),
            SmtpStream::Tls(_) => {
                self.write_response(503, "TLS already active").await?;
                return Ok(());
            }
            SmtpStream::Placeholder => unreachable!(),
        };

        let tls_stream = acceptor.accept(plain_stream).await?;

        self.stream = SmtpStream::Tls(BufReader::new(tls_stream));
        self.handle_reset().await?;
        Ok(())
    }

    async fn handle_helo(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            self.write_response(501, "Syntax error in parameters or arguments")
                .await?;
            return Ok(());
        }
        self.smtp_helo = Some(parts[1].to_string());
        self.helo_seen = true;
        self.write_response(250, &format!("Hello {}", parts[1]))
            .await?;
        Ok(())
    }

    async fn handle_ehlo(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            self.write_response(501, "Syntax error in parameters or arguments")
                .await?;
            return Ok(());
        }
        self.smtp_helo = Some(parts[1].to_string());
        self.helo_seen = true;
        let size_cap = format!("SIZE {}", self.max_message_size);
        self.write_multiline_response(
            250,
            &[
                &format!("Hello {}", parts[1]),
                &size_cap,
                "SMTPUTF8",
                "8BITMIME",
                "PIPELINING",
                if self.tls_acceptor.is_some() {
                    "STARTTLS"
                } else {
                    ""
                },
            ],
        )
        .await?;
        Ok(())
    }

    async fn handle_mail(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.helo_seen {
            self.write_response(503, "Bad sequence of commands").await?;
            return Ok(());
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 || !parts[1].to_uppercase().starts_with("FROM:") {
            self.write_response(501, "Syntax error in parameters or arguments")
                .await?;
            return Ok(());
        }
        match parts[1][5..]
            .trim_start_matches("<")
            .trim_end_matches(">")
            .parse::<Mailbox>()
        {
            Ok(mailbox) => {
                if !mailbox.display_name.is_none() || mailbox.address().is_empty() {
                    self.write_response(501, "Invalid email address").await?;
                    return Ok(());
                }
                self.mail_from = Some(mailbox);
                self.mail_seen = true;
                self.write_response(250, "OK").await?;
            }
            Err(_) => {
                self.write_response(501, "Invalid email address").await?;
            }
        }
        Ok(())
    }

    async fn handle_rcpt(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mail_seen {
            self.write_response(503, "Bad sequence of commands").await?;
            return Ok(());
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 || !parts[1].to_uppercase().starts_with("TO:") {
            self.write_response(501, "Syntax error in parameters or arguments")
                .await?;
            return Ok(());
        }
        match parts[1][3..]
            .trim_start_matches("<")
            .trim_end_matches(">")
            .parse::<Mailbox>()
        {
            Ok(mailbox) => {
                if !mailbox.display_name.is_none() || mailbox.address().is_empty() {
                    self.write_response(501, "Invalid email address").await?;
                    return Ok(());
                }
                self.rcpt_to.push(mailbox);
                self.rcpt_seen = true;
                self.write_response(250, "OK").await?;
            }
            Err(_) => {
                self.write_response(501, "Invalid email address").await?;
            }
        }
        Ok(())
    }

    async fn handle_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.rcpt_seen {
            self.write_response(503, "Bad sequence of commands").await?;
            return Ok(());
        }
        self.write_response(354, "End data with <CR><LF>.<CR><LF>")
            .await?;
        let mut data = Vec::new();
        let mut size = 0usize;
        loop {
            let line = self.read_line().await?;
            size += line.len() + 2; // +2 for CRLF
            if size > self.max_message_size {
                self.write_response(552, "Message size exceeds fixed maximum message size")
                    .await?;
                self.handle_reset().await?;
                return Ok(());
            }
            if line == "." {
                break;
            }
            data.extend_from_slice(line.as_bytes());
            data.extend_from_slice(b"\r\n");
        }
        self.message_data = Some(data);
        if let Some(handler) = &*self.message_handler {
            handler.on_message(
                SmtpEnvelope::new(
                    self.smtp_helo.clone().unwrap(),
                    self.mail_from.clone().unwrap(),
                    self.rcpt_to.clone(),
                ),
                self.message_data.clone().unwrap_or_default(),
            )?;
        }
        self.write_response(250, "OK").await?;
        Ok(())
    }

    async fn handle_reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.mail_from = None;
        self.rcpt_to.clear();
        self.message_data = None;
        self.mail_seen = false;
        self.rcpt_seen = false;
        self.write_response(250, "OK").await?;
        Ok(())
    }

    async fn write_response(
        &mut self,
        code: u16,
        msg: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = format!("{} {}\r\n", code, msg);
        timeout(
            Duration::from_secs(self.write_timeout),
            self.stream.write_all(response.as_bytes()),
        )
        .await??;
        Ok(())
    }

    #[allow(dead_code)]
    async fn write_enhanced_response(
        &mut self,
        code: u16,
        enhanced_code: &str,
        msg: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = format!("{} {} {}\r\n", code, enhanced_code, msg);
        self.stream.write_all(response.as_bytes()).await?;
        Ok(())
    }

    async fn write_multiline_response(
        &mut self,
        code: u16,
        lines: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (i, line) in lines.iter().enumerate() {
            let separator = if i == lines.len() - 1 { " " } else { "-" };
            let response = format!("{}{}{}\r\n", code, separator, line);
            self.stream.write_all(response.as_bytes()).await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    async fn write_multiline_enhanced_response(
        &mut self,
        code: u16,
        enhanced_code: &str,
        lines: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (i, line) in lines.iter().enumerate() {
            let separator = if i == lines.len() - 1 { " " } else { "-" };
            let response = format!("{}{}{} {}\r\n", code, separator, enhanced_code, line);
            self.stream.write_all(response.as_bytes()).await?;
        }
        Ok(())
    }

    async fn read_line(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        timeout(
            Duration::from_secs(self.read_timeout),
            self.stream.read_line(&mut buffer),
        )
        .await??;
        Ok(buffer.trim_end().to_string())
    }
}
