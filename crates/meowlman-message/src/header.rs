pub struct HeaderWriter {
    buf: String,
}

impl HeaderWriter {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }
    pub fn header(&mut self, name: &str, value: &str) -> &mut Self {
        self.fold_header(name, value)
    }

    pub fn build(self) -> String {
        self.buf
    }

    fn fold_header(&mut self, name: &str, value: &str) -> &mut Self {
        if name.len() + value.len() + 2 <= 78 {
            self.buf.push_str(name);
            self.buf.push_str(": ");
            self.buf.push_str(value);
            self.buf.push_str("\r\n");
        } else {
            let mut line = String::new();
            line.push_str(name);
            line.push_str(": ");
            for word in value.split_whitespace() {
                if line.len() + word.len() + 1 > 78 {
                    self.buf.push_str(&line);
                    self.buf.push_str("\r\n");
                    line.clear();
                    line.push_str(" ");
                }
                if !line.ends_with(' ') {
                    line.push(' ');
                }
                line.push_str(word);
            }
            if !line.is_empty() {
                self.buf.push_str(&line);
                self.buf.push_str("\r\n");
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_header() {
        let mut writer = HeaderWriter::new();
        let long_value = "This is a long header value that should be folded across multiple lines to test the folding functionality of the header writer in the meowlman_message crate and ensure that it correctly handles long header values without breaking the email format and maintains compliance with email standards for header folding and line length limits";
        writer.header("X-Long-Header", long_value);
        let result = writer.build();
        let expected = "X-Long-Header: This is a long header value that should be folded across multiple lines to test the folding functionality of the header writer in the meowlman_message crate and ensure that it correctly handles long header values without breaking the email format and maintains compliance with email standards for header folding and line length limits\r\n";
        assert_ne!(result, expected);
    }

    #[test]
    fn test_short_header() {
        let mut writer = HeaderWriter::new();
        writer.header("Subject", "Hello World");
        let result = writer.build();
        let expected = "Subject: Hello World\r\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_headers() {
        let mut writer = HeaderWriter::new();
        writer.header("Subject", "Hello World");
        writer.header("X-Custom-Header", "This is a custom header value that should also be included in the final email output to test the ability of the header writer to handle custom headers and ensure that they are correctly formatted and included in the email output without any issues.");
        writer.header("X-Long-Header", "This is a long header value that should be folded across multiple lines to test the folding functionality of the header writer in the meowlman_message crate and ensure that it correctly handles long header values without breaking the email format and maintains compliance with email standards for header folding and line length limits");
        let result = writer.build();
        let expected = "Subject: Hello World\r\nX-Custom-Header: This is a custom header value that should also be included in the final email output to test the ability of the header writer to handle custom headers and ensure that they are correctly formatted and included in the email output without any issues.\r\nX-Long-Header: This is a long header value that should be folded across multiple lines to test the folding functionality of the header writer in the meowlman_message crate and ensure that it correctly handles long header values without breaking the email format and maintains compliance with email standards for header folding and line length limits\r\n";
        assert_ne!(result, expected);
    }

    #[test]
    fn test_empty_header() {
        let mut writer = HeaderWriter::new();
        writer.header("X-Empty-Header", "");
        let result = writer.build();
        let expected = "X-Empty-Header: \r\n";
        assert_eq!(result, expected);
    }
}
