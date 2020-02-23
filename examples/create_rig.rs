use meme_machine_database::*;
use std::io::{stdin, Read};

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";

fn main() {
    let connection = establish_connection();

    let mut title = String::new();
    let mut body = String::new();

    println!("What would you like your new rig's title to be?");
    stdin()
        .read_line(&mut title)
        .expect("Please enter a title for the Rig");
    let title = title.trim();

    println!(
        "\nOk! Please write a description of Rig {} (Start a new line and press {} when finished)\n",
        title, EOF
    );
    stdin()
        .read_to_string(&mut body)
        .expect("Please enter a description for the Rig");
    let body = body.trim();

    let post = create_rig(&connection, &title, &body);
    println!("\nSaved draft {} with id {}", title, post.id);
    println!("\n{}", body);
}
