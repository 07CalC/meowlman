use meowlman_address::Mailbox;

pub struct SmtpEnvelope {
    pub helo_name: String,
    pub from: Mailbox,
    pub to: Vec<Mailbox>,
}

impl SmtpEnvelope {
    pub fn new(helo_name: String, from: Mailbox, to: Vec<Mailbox>) -> Self {
        SmtpEnvelope {
            helo_name,
            from,
            to,
        }
    }
}
