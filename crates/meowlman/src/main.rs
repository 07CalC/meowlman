use meowlman_message::{Attachment, Message};
fn main() {
    let message = Message::new("hello@vinm.me".parse().unwrap())
        .to("maheshwarivinayak90@gmail.com".parse().unwrap())
        .to("kysvin@mortar.email".parse().unwrap())
        .subject("hi")
        .body_text("hi")
        .body_html("<h1>hi</h1>")
        .attachment(Attachment::from_file("resume-v1.5.pdf", "application/pdf").unwrap());
    let mut client = meowlman_smtp::SmtpClient::connect("localhost:1025").unwrap_or_else(|e| {
        eprintln!("Failed to connect to SMTP server: {}", e);
        std::process::exit(1);
    });
    client
        .send(
            &"hello@vinm.me".parse().unwrap(),
            &vec!["maheshwarivinayak90@gmail.com".parse().unwrap()],
            &message,
        )
        .unwrap();
}
