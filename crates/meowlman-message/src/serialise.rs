use std::hash::{BuildHasher, Hasher, RandomState};

use crate::{Message, header::HeaderWriter};

pub struct MessageFormatter {}

impl MessageFormatter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn format(message: &Message) -> String {
        let mut header_writer = HeaderWriter::new();
        header_writer.header(
            "From",
            &message
                .from
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
        if let Some(ref sender) = message.sender {
            header_writer.header("Sender", &sender.to_string());
        }
        if !message.to.is_empty() {
            header_writer.header(
                "To",
                &message
                    .to
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if !message.cc.is_empty() {
            header_writer.header(
                "Cc",
                &message
                    .cc
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if let Some(ref subject) = message.subject {
            header_writer.header("Subject", subject);
        }
        if let Some(ref reply_to) = message.reply_to {
            header_writer.header(
                "Reply-To",
                &reply_to
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if let Some(ref in_reply_to) = message.in_reply_to {
            header_writer.header("In-Reply-To", in_reply_to);
        }
        if !message.references.is_empty() {
            let references_str = message.references.join(" ");
            header_writer.header("References", &references_str);
        }
        if let Some(ref date) = message.date {
            header_writer.header("Date", date);
        } else {
            header_writer.header("Date", &chrono::Utc::now().to_rfc2822());
        }
        if let Some(ref message_id) = message.message_id {
            header_writer.header("Message-ID", message_id);
        } else {
            let generated_id = format!(
                "<{}@{}>",
                chrono::Utc::now().timestamp_millis(),
                message.from[0].address()
            );
            header_writer.header("Message-ID", &generated_id);
        }
        for (name, value) in &message.headers {
            header_writer.header(name, value);
        }

        let mut result = header_writer.build();
        result.push_str("\r\n");
        if let Some(ref body_text) = message.body_text {
            result.push_str(body_text);
        } else if let Some(ref body_html) = message.body_html {
            result.push_str(body_html);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let message = Message::new("something@email.com".parse().unwrap())
            .to("somethingelse@email.com".parse().unwrap())
            .subject("Test Email")
            .body_text("This is a test email.");
        let formatted = MessageFormatter::format(&message);
        assert!(formatted.contains("From: something@email.com"));
        assert!(formatted.contains("To: somethingelse@email.com"));
        assert!(formatted.contains("Subject: Test Email"));
        assert!(formatted.contains("This is a test email."));
    }

    #[test]
    fn test_format_message_with_custom_headers() {
        let message = Message::new("something@email.com".parse().unwrap())
            .to("somethingelse@email.com".parse().unwrap())
            .subject("Test Email")
            .body_text("This is a test email.")
            .header("X-Custom-Header", "Custom Value");
        let formatted = MessageFormatter::format(&message);
        assert!(formatted.contains("X-Custom-Header: Custom Value"));
    }
}
