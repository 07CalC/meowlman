fn main() {
    // Parse from string
    let mb = meowlman_address::Mailbox::from_str("John Doe <john.doe@gmail.com>").unwrap();
    println!("parsed:  {mb}");
    println!("  display_name: {:?}", mb.display_name);
    println!("  local_part:   {:?}", mb.local_part);
    println!("  domain:       {:?}", mb.domain);

    // Construct from parts
    let mb = meowlman_address::Mailbox::new(Some("Jane Doe".into()), "jane.doe", "example.com");
    println!("built:   {mb}");
    println!("  display_name: {:?}", mb.display_name);
    println!("  local_part:   {:?}", mb.local_part);
    println!("  domain:       {:?}", mb.domain);

    // Bare address
    let mb = meowlman_address::Mailbox::new(None, "robot", "service.org");
    println!("bare:    {mb}");
    println!("  display_name: {:?}", mb.display_name);
    println!("  local_part:   {:?}", mb.local_part);
    println!("  domain:       {:?}", mb.domain);
}
