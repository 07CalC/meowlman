use meowlman::{Mailbox, SmtpClient};

fn main() {
    let mut client = SmtpClient::connect("localhost:2525").expect("connect");
    let resp = client.helo("example.com").expect("EHLO");
    println!("EHLO: {} {:?}", resp.code, resp.lines);

    let mailbox = Mailbox::from_str("vinayak <hello@vinm.me>").unwrap();
    let resp = client.mail_from(mailbox).expect("MAIL FROM");
    println!("MAIL FROM: {} {:?}", resp.code, resp.lines);

    let resp = client
        .rcpt_to(Mailbox::from_str("calc <calc@trymist.cloud>").unwrap())
        .expect("RCPT TO");
    println!("RCPT TO: {} {:?}", resp.code, resp.lines);

    match client.data("Subject: Test\r\n\r\nThis is a test email.") {
        Ok(resp) => println!("DATA: {} {:?}", resp.code, resp.lines),
        Err(e) => eprintln!("DATA failed: {e}"),
    }
}
