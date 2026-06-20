use poseidon::store::memory::Store;
use std::io::{self, Read, Write};

enum Command {
    Get {key : String},
    Put {key : String, value : String},
    Delete { key: String}
}

fn main() {
    loop {
        print!("Welcome to Poseidon! ");
        io::stdout().flush().unwrap_err();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let clean_input = input.trim();
        let mut words = clean_input.split_whitespace();

        let command_name = words.next();
        let key = words.next();
        let value = words.next();

        if input.trim() == "exit" {
            break;
        }
    }
}
