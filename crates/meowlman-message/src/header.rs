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
