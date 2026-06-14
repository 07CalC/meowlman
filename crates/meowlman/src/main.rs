use meowlman_message::{Attachment, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::new("hello@vinm.me".parse().unwrap())
        .to("maheshwarivinayak90@gmail.com".parse().unwrap())
        .to("kysvin@mortar.email".parse().unwrap())
        .subject("hi")
        .body_text("hi")
        .body_html("<h1>hi</h1>");

    let mut server = meowlman_smtp::SmtpServer::new("0.0.0.0".to_string(), 1025)
        .with_message_handler(Box::new(MessageHandler))
        .with_helo_name("my-smtp-server".to_string())
        .with_start_tls("./cert.pem", "./key.pem")?;

    if let Err(e) = server.serve().await {
        eprintln!("SMTP server error: {}", e);
    }

    Ok(())
}

struct MessageHandler;
impl meowlman_smtp::MessageHandler for MessageHandler {
    fn on_message(
        &self,
        envelope: meowlman_smtp::SmtpEnvelope,
        message: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "helo: {}, from: {}, to: {:?}\nmessage: {} ",
            envelope.helo_name,
            envelope.from.to_string(),
            envelope
                .to
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>(),
            String::from_utf8(message).unwrap_or_else(|_| "<invalid utf-8>".to_string())
        );

        Ok(())
    }
}
