use meowlman_message::{Attachment, Message};
fn main() {
    let message = Message::new("hello@vinm.me".parse().unwrap())
        .to("maheshwarivinayak90@gmail.com".parse().unwrap())
        .to("kysvin@mortar.email".parse().unwrap())
        .subject("hi")
        .body_text("hi")
        .body_html("<h1>hi</h1>")
        .attachment(Attachment::from_file("hi.txt", "text/plain").unwrap())
        .attachment(Attachment::from_file("resume-v1.5.pdf", "application/pdf").unwrap());
    let mut client = meowlman_smtp::SmtpClient::connect("localhost:1025").unwrap();
    let mut res = client.helo("localhost").unwrap();
    println!("HELO response: {:?}", res);
    res = client.mail_from("hello@vinm.me".parse().unwrap()).unwrap();
    println!("MAIL FROM response: {:?}", res);
    res = client.rcpt_to("mailhog@vinm.me".parse().unwrap()).unwrap();
    println!("RCPT TO response: {:?}", res);
    res = client.send(&message).unwrap();
    println!("DATA response: {:?}", res);
}
