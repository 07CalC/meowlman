pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

impl Attachment {
    pub fn new(filename: &str, content_type: &str, content: Vec<u8>) -> Self {
        Self {
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            content,
        }
    }
    pub fn from_file(path: &str, content_type: &str) -> std::io::Result<Self> {
        let content = std::fs::read(path)?;
        let filename = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        Ok(Self {
            filename,
            content_type: content_type.to_string(),
            content,
        })
    }
}
