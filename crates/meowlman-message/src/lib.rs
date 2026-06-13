use meowlman_address::Mailbox;
mod header;
mod mime;

mod serialise;

pub use serialise::MessageFormatter;

/// ref: https://datatracker.ietf.org/doc/html/rfc5322#section-3.6
//  +----------------+--------+------------+----------------------------+
// | Field          | Min    | Max number | Notes                      |
// |                | number |            |                            |
// +----------------+--------+------------+----------------------------+
// | trace          | 0      | unlimited  | Block prepended - see      |
// |                |        |            | 3.6.7                      |
// | resent-date    | 0*     | unlimited* | One per block, required if |
// |                |        |            | other resent fields are    |
// |                |        |            | present - see 3.6.6        |
// | resent-from    | 0      | unlimited* | One per block - see 3.6.6  |
// | resent-sender  | 0*     | unlimited* | One per block, MUST occur  |
// |                |        |            | with multi-address         |
// |                |        |            | resent-from - see 3.6.6    |
// | resent-to      | 0      | unlimited* | One per block - see 3.6.6  |
// | resent-cc      | 0      | unlimited* | One per block - see 3.6.6  |
// | resent-bcc     | 0      | unlimited* | One per block - see 3.6.6  |
// | resent-msg-id  | 0      | unlimited* | One per block - see 3.6.6  |
// | orig-date      | 1      | 1          |                            |
// | from           | 1      | 1          | See sender and 3.6.2       |
// | sender         | 0*     | 1          | MUST occur with            |
// |                |        |            | multi-address from - see   |
// |                |        |            | 3.6.2                      |
// | reply-to       | 0      | 1          |                            |
// | to             | 0      | 1          |                            |
// | cc             | 0      | 1          |                            |
// | bcc            | 0      | 1          |                            |
// | message-id     | 0*     | 1          | SHOULD be present - see    |
// |                |        |            | 3.6.4                      |
// | in-reply-to    | 0*     | 1          | SHOULD occur in some       |
// |                |        |            | replies - see 3.6.4        |
// | references     | 0*     | 1          | SHOULD occur in some       |
// |                |        |            | replies - see 3.6.4        |
// | subject        | 0      | 1          |                            |
// | comments       | 0      | unlimited  |                            |
// | keywords       | 0      | unlimited  |                            |
// | optional-field | 0      | unlimited  |                            |
// +----------------+--------+------------+----------------------------+

//TODO: add support for attachments and multipart messags
pub struct Message {
    pub from: Vec<Mailbox>,
    pub sender: Option<Mailbox>,
    pub to: Vec<Mailbox>,
    pub cc: Vec<Mailbox>,
    pub bcc: Vec<Mailbox>,
    pub subject: Option<String>,
    pub reply_to: Option<Vec<Mailbox>>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub message_id: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl Message {
    pub fn new(from: Mailbox) -> Self {
        Self {
            from: vec![from],
            sender: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: None,
            reply_to: None,
            in_reply_to: None,
            references: Vec::new(),
            date: None,
            body_text: None,
            body_html: None,
            message_id: None,
            headers: Vec::new(),
        }
    }
    pub fn sender(mut self, sender: Mailbox) -> Self {
        self.sender = Some(sender);
        self
    }
    pub fn to(mut self, to: Mailbox) -> Self {
        self.to.push(to);
        self
    }

    pub fn cc(mut self, cc: Mailbox) -> Self {
        self.cc.push(cc);
        self
    }
    pub fn bcc(mut self, bcc: Mailbox) -> Self {
        self.bcc.push(bcc);
        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }
    pub fn reply_to(mut self, reply_to: Mailbox) -> Self {
        if let Some(ref mut existing) = self.reply_to {
            existing.push(reply_to);
        } else {
            self.reply_to = Some(vec![reply_to]);
        }
        self
    }
    pub fn in_reply_to(mut self, in_reply_to: &str) -> Self {
        self.in_reply_to = Some(in_reply_to.to_string());
        self
    }
    pub fn references(mut self, reference: &str) -> Self {
        self.references.push(reference.to_string());
        self
    }
    pub fn date(mut self, date: &str) -> Self {
        self.date = Some(date.to_string());
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn body_text(mut self, body_text: &str) -> Self {
        self.body_text = Some(body_text.to_string());
        self
    }
    pub fn body_html(mut self, body_html: &str) -> Self {
        self.body_html = Some(body_html.to_string());
        self
    }

    pub fn message_id(mut self, message_id: &str) -> Self {
        self.message_id = Some(message_id.to_string());
        self
    }

    pub fn build(&self) -> String {
        MessageFormatter::format(&self)
    }
}
