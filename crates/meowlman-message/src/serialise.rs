use crate::{
    Message,
    header::HeaderWriter,
    mime::{Encoding, MimeNode, MimePart, MultipartKind},
};

pub struct MessageFormatter {}

impl MessageFormatter {
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
        //TODO: handle multipart messages with both text and html bodies, for now just include one
        //or the other if present
        let body_node = Self::build_tree(message);
        result.push_str(&body_node.build());
        result
    }

    fn build_tree(message: &Message) -> MimeNode {
        let mut content = match (&message.body_text, &message.body_html) {
            (Some(text), Some(html)) => MimeNode::Multipart {
                kind: MultipartKind::Alternative,
                parts: vec![
                    MimeNode::Part(MimePart {
                        content_type: "text/plain; charset=utf-8".to_string(),
                        content_transfer_encoding: Encoding::QuotedPrintable,
                        content_disposition: None,
                        content: text.as_bytes().to_vec(),
                        headers: Vec::new(),
                    }),
                    MimeNode::Part(MimePart {
                        content_type: "text/html; charset=utf-8".to_string(),
                        content_transfer_encoding: Encoding::QuotedPrintable,
                        content_disposition: None,
                        content: html.as_bytes().to_vec(),
                        headers: Vec::new(),
                    }),
                ],
            },
            (Some(text), None) => MimeNode::Part(MimePart {
                content_type: "text/plain; charset=utf-8".to_string(),
                content_transfer_encoding: Encoding::QuotedPrintable,
                content_disposition: None,
                content: text.as_bytes().to_vec(),
                headers: Vec::new(),
            }),
            (None, Some(html)) => MimeNode::Part(MimePart {
                content_type: "text/html; charset=utf-8".to_string(),
                content_transfer_encoding: Encoding::QuotedPrintable,
                content_disposition: None,
                content: html.as_bytes().to_vec(),
                headers: Vec::new(),
            }),
            (None, None) => MimeNode::Part(MimePart {
                content_type: "text/plain; charset=utf-8".to_string(),
                content_transfer_encoding: Encoding::QuotedPrintable,
                content_disposition: None,
                content: Vec::new(),
                headers: Vec::new(),
            }),
        };
        return content;
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

    #[test]
    fn test_format_message_with_html_body() {
        let message = Message::new("vinayak <hello@vinm.me>".parse().unwrap())
            .to("calc <hello@calc.me>".parse().unwrap())
            .subject("Test Email with HTML")
            .body_html("<h1>This is a test email.</h1>");
        let formatted = MessageFormatter::format(&message);
        assert!(formatted.contains("Content-Type: text/html; charset=utf-8"));
        assert!(formatted.contains("<h1>This is a test email.</h1>"));
    }

    #[test]
    fn test_format_message_with_both_bodies() {
        let message = Message::new("vinayak <hello@vinm.me>".parse().unwrap())
            .to("calc <hello@calc.me>".parse().unwrap())
            .subject("Test Email with Both Bodies")
            .body_text("This is the plain text body.")
            .body_html("<h1>This is the HTML body.</h1>");
        let formatted = MessageFormatter::format(&message);
        assert!(formatted.contains("Content-Type: multipart/alternative"));
        assert!(formatted.contains("Content-Type: text/plain; charset=utf-8"));
        assert!(formatted.contains("Content-Type: text/html; charset=utf-8"));
        assert!(formatted.contains("This is the plain text body."));
        assert!(formatted.contains("<h1>This is the HTML body.</h1>"));
    }
}
