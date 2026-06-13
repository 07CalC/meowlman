use meowlman_message::{Attachment, Message};
fn main() {
    let message = Message::new("hello@vinm.me".parse().unwrap())
    .to("maheshwarivinayak90@gmail.com".parse().unwrap())
    .to("kysvin@mortar.email".parse().unwrap())
    .subject("hello this is a long subject that should be folded across multiple lines to test the folding functionality of the message formatter in the meowlman_message crate and ensure that it correctly handles long header values without breaking the email format and maintains compliance with email standards for header folding and line length limits")
    .body_text("This is the body of the email. It can contain multiple lines of;multiple text to test the body formatting functionality of the message formatter in the meowlman_message crate. The body should be correctly formatted and included in the final email output without any issues.")
    .body_html("<h1>This is the HTML body of the email.</h1><p>It can contain multiple lines of HTML content to test the body formatting functionality of the message formatter in the meowlman_message crate. The HTML body should be correctly formatted and included in the final email output without any issues.</p>")
    .header("X-Custom-Header", "This is a custom header value that should also be included in the final email output to test the ability of the message formatter to handle custom headers and ensure that they are correctly formatted and included in the email output without any issues.").
    attachment(Attachment::from_file("hi.txt", "text/plain").unwrap());
    let serialised = message.build();
    println!("{}", serialised);
}
