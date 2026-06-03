use meowlman_address;

fn main() {
    let mailbox = meowlman_address::Mailbox::new("john doe <john.doe@gmail.com>").unwrap();
    println!("{:?}", mailbox.display_name.unwrap());
    println!("{:?}", mailbox.addr_spec.address());
}
