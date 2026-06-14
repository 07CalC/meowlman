use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use meowlman_address::Mailbox;
use meowlman_message::Message;

use crate::error::SmtpClientError;

#[derive(Debug)]
pub struct Response {
    pub code: u16,
    pub lines: Vec<String>,
}

impl Response {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code)
    }

    pub fn check(&self, expected: u16) -> Result<(), SmtpClientError> {
        if self.code == expected {
            Ok(())
        } else {
            Err(SmtpClientError::UnexpectedResponse {
                expected: expected.to_string(),
                code: self.code,
                detail: self.lines.join("\n"),
            })
        }
    }

    pub fn check_2xx(&self) -> Result<(), SmtpClientError> {
        if self.is_success() {
            Ok(())
        } else {
            Err(SmtpClientError::UnexpectedResponse {
                expected: "2xx".to_string(),
                code: self.code,
                detail: self.lines.join("\n"),
            })
        }
    }

    fn check_any(&self, expected: &[u16]) -> Result<(), SmtpClientError> {
        if expected.contains(&self.code) {
            Ok(())
        } else {
            let list = expected
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("/");
            Err(SmtpClientError::UnexpectedResponse {
                expected: list,
                code: self.code,
                detail: self.lines.join("\n"),
            })
        }
    }
}

/// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.2
/// `Before sending a line of mail text, the SMTP client checks the
/// first character of the line.  If it is a period, one additional
/// period is inserted at the beginning of the line.`
fn prepare_data_body(body: &str) -> String {
    let mut out = String::with_capacity(body.len() + 128);
    for raw in body.split('\n') {
        let line = raw.trim_end_matches('\r');
        if line.starts_with('.') {
            out.push_str("..");
            out.push_str(&line[1..]);
        } else {
            out.push_str(line);
        }
        out.push_str("\r\n");
    }
    out
}

pub struct SmtpClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl SmtpClient {
    pub fn connect(addr: &str) -> Result<Self, SmtpClientError> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        let reader = BufReader::new(stream.try_clone()?);
        let mut client = Self { stream, reader };
        let greeting = client.read_response()?;
        greeting.check(220)?;
        Ok(client)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.1
    pub fn helo(&mut self, domain: &str) -> Result<Response, SmtpClientError> {
        let resp = self.command(&format!("EHLO {}", domain))?;
        resp.check_2xx()?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.2
    pub fn mail_from(&mut self, mb: &Mailbox) -> Result<Response, SmtpClientError> {
        let resp = self.command(&format!("MAIL FROM:<{}>", mb.address()))?;
        resp.check_2xx()?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.3
    pub fn rcpt_to(&mut self, mb: &Mailbox) -> Result<Response, SmtpClientError> {
        let resp = self.command(&format!("RCPT TO:<{}>", mb.address()))?;
        resp.check_any(&[250, 251])?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.4
    pub fn data_raw(&mut self, body: &str) -> Result<Response, SmtpClientError> {
        let resp = self.command("DATA")?;
        resp.check(354)?;

        let stuffed = prepare_data_body(body);
        self.stream.write_all(stuffed.as_bytes())?;
        self.stream.write_all(b".\r\n")?;
        self.stream.flush()?;

        let resp = self.read_response()?;
        resp.check_2xx()?;
        Ok(resp)
    }

    pub fn msg(&mut self, msg: &Message) -> Result<Response, SmtpClientError> {
        let resp = self.data_raw(&msg.build())?;
        Ok(resp)
    }

    pub fn send(
        &mut self,
        from: &Mailbox,
        to: &[Mailbox],
        msg: &Message,
    ) -> Result<Response, SmtpClientError> {
        self.helo("localhost")?;
        self.mail_from(from)?;
        for rcpt in to {
            self.rcpt_to(rcpt)?;
        }
        let resp = self.msg(msg)?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.5
    pub fn rset(&mut self) -> Result<Response, SmtpClientError> {
        let resp = self.command("RSET")?;
        resp.check_2xx()?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.10
    pub fn quit(&mut self) -> Result<Response, SmtpClientError> {
        let resp = self.command("QUIT")?;
        resp.check_2xx()?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.9
    pub fn noop(&mut self) -> Result<Response, SmtpClientError> {
        let resp = self.command("NOOP")?;
        resp.check_2xx()?;
        Ok(resp)
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1
    /// `SMTP commands are character strings
    /// terminated by <CRLF>.  The commands themselves are alphabetic
    /// characters terminated by <SP> if parameters follow and <CRLF>
    /// otherwise.`
    fn command(&mut self, cmd: &str) -> Result<Response, SmtpClientError> {
        self.stream.write_all(cmd.as_bytes())?;
        self.stream.write_all(b"\r\n")?;
        self.stream.flush()?;
        self.read_response()
    }

    /// ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.2
    /// `An SMTP reply consists of a three digit number (transmitted as three
    /// numeric characters) followed by some text unless specified otherwise
    /// in this document.`
    fn read_response(&mut self) -> Result<Response, SmtpClientError> {
        let mut lines = Vec::new();
        let mut code: Option<u16> = None;

        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                return Err(SmtpClientError::ConnectionClosed);
            }

            if line.len() < 4 {
                return Err(SmtpClientError::InvalidResponse(format!(
                    "response line too short: {line:?}"
                )));
            }

            let line_code: u16 = line[..3].parse().map_err(|_| {
                SmtpClientError::InvalidResponse(format!("malformed code in: {line:?}"))
            })?;

            match code {
                None => code = Some(line_code),
                Some(prev) if prev != line_code => {
                    return Err(SmtpClientError::InconsistentCodes {
                        first: prev,
                        second: line_code,
                    });
                }
                _ => {}
            }

            let text = line[4..].trim_end_matches(&['\r', '\n'][..]);
            lines.push(text.to_string());

            // ref: https://datatracker.ietf.org/doc/html/rfc5321#section-4.2.1
            // `The format for multiline replies requires that every line, except the
            // last, begin with the reply code, followed immediately by a hyphen,
            // "-" (also known as minus), followed by text.  The last line will
            // begin with the reply code, followed immediately by <SP>, optionally
            // some text, and <CRLF>.  As noted above, servers SHOULD send the <SP>
            // if subsequent text is not sent, but clients MUST be prepared for it
            // to be omitted.`

            // `For example:

            // 250-First line
            // 250-Second line
            // 250-234 Text beginning with numbers
            // 250 The last line`
            if line.as_bytes()[3] == b' ' {
                break;
            }
        }

        Ok(Response {
            code: code.unwrap_or(0),
            lines,
        })
    }
}
