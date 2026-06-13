use base64::{Engine, engine};

pub enum MimeNode {
    Part(MimePart),
    Multipart {
        kind: MultipartKind,
        parts: Vec<MimeNode>,
    },
}

pub enum MultipartKind {
    Mixed,
    Alternative,
    Related,
}

pub struct MimePart {
    pub content_type: String,
    pub content_transfer_encoding: Encoding,
    pub content_disposition: Option<String>,
    pub content: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

pub enum Encoding {
    SevenBit,
    EightBit,
    Binary,
    Base64,
    QuotedPrintable,
}

impl MimeNode {
    pub fn is_multipart(&self) -> bool {
        matches!(self, MimeNode::Multipart { .. })
    }
    pub fn is_part(&self) -> bool {
        matches!(self, MimeNode::Part(_))
    }

    pub fn build(&self) -> String {
        match self {
            MimeNode::Part(part) => {
                let mut result = String::new();
                result.push_str(&format!("Content-Type: {}\r\n", part.content_type));
                result.push_str(&format!(
                    "Content-Transfer-Encoding: {}\r\n",
                    match part.content_transfer_encoding {
                        Encoding::SevenBit => "7bit",
                        Encoding::EightBit => "8bit",
                        Encoding::Binary => "binary",
                        Encoding::Base64 => "base64",
                        Encoding::QuotedPrintable => "quoted-printable",
                    }
                ));
                if let Some(disposition) = &part.content_disposition {
                    result.push_str(&format!("Content-Disposition: {}\r\n", disposition));
                }
                for (name, value) in &part.headers {
                    result.push_str(&format!("{}: {}\r\n", name, value));
                }
                result.push_str("\r\n");
                let content_str = match part.content_transfer_encoding {
                    Encoding::Base64 => engine::general_purpose::STANDARD.encode(&part.content),
                    Encoding::QuotedPrintable => {
                        String::from_utf8(quoted_printable::encode(&part.content)).unwrap()
                    }
                    _ => String::from_utf8_lossy(&part.content).to_string(),
                };
                result.push_str(&content_str);
                result
            }
            MimeNode::Multipart { kind, parts } => {
                let boundary = format!("boundary_{}", chrono::Utc::now().timestamp_subsec_nanos());
                let mut result = String::new();
                result.push_str(&format!(
                    "Content-Type: multipart/{}; boundary=\"{}\"\r\n\r\n",
                    match kind {
                        MultipartKind::Mixed => "mixed",
                        MultipartKind::Alternative => "alternative",
                        MultipartKind::Related => "related",
                    },
                    boundary
                ));
                for part in parts {
                    result.push_str(&format!("--{}\r\n", boundary));
                    result.push_str(&part.build());
                    result.push_str("\r\n");
                }
                result.push_str(&format!("--{}--\r\n", boundary));
                result
            }
        }
    }
}

impl MimePart {
    pub fn new_text(content: &str) -> Self {
        MimePart {
            content_type: "text/plain; charset=utf-8".to_string(),
            content_transfer_encoding: Encoding::QuotedPrintable,
            content_disposition: None,
            content: content.as_bytes().to_vec(),
            headers: vec![],
        }
    }
    pub fn new_html(content: &str) -> Self {
        MimePart {
            content_type: "text/html; charset=utf-8".to_string(),
            content_transfer_encoding: Encoding::QuotedPrintable,
            content_disposition: None,
            content: content.as_bytes().to_vec(),
            headers: vec![],
        }
    }

    pub fn new_attachment(filename: &str, content_type: &str, content: Vec<u8>) -> Self {
        MimePart {
            content_type: content_type.to_string(),
            content_transfer_encoding: Encoding::Base64,
            content_disposition: Some(format!("attachment; filename=\"{}\"", filename)),
            content,
            headers: vec![],
        }
    }
}
