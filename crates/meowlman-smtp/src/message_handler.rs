use crate::envelope::SmtpEnvelope;

pub trait MessageHandler: Send + Sync {
    fn on_message(
        &self,
        envelope: SmtpEnvelope,
        message: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
