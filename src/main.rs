use poseidon::store::memory::Store;
use std::io::{self, Read, Write};

enum Command {
    Get {key : String},
    Put {key : String, value : String},
    Delete { key: String}
}

fn main() {
    let mut start_store = Store::new();
    loop {
        print!("Welcome to Poseidon! ");
        io::stdout().flush().unwrap();

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

        match command_name {
            Some("Get") => {
                 if let Some(k) = key {
                    let _cmd = Command::Get { key: k.to_string() };
                    match _cmd {
                        Command::Get { key } => {
                            println!("Executing Get Operation For : {}", key);
                            match start_store.get(key) {
                                Some(value) => println!("Value : {}", value),
                                None => println!("Key not found"),
                            }
                        }
                        _ => {}
                    }
                 } else {
                    println!("You must provide a key value");
                 }
            }
            Some("Put") => {
                if let (Some(k),Some(v)) = (key,value) {
                    let _cmd = Command::Put{key:k.to_string(), value:v.to_string()};
                    match _cmd {
                        Command::Put {key , value} => {
                            println!("Executing Put Operation For : {} , Value : {}", key , value);
                            start_store.put(key, value);
                        }
                        _ => {}
                    }
                } else {
                    println!("You must provide a key value and a value");
                }
            }
            Some("Delete") => {
                if let Some(k) = key {
                    let _cmd = Command::Delete{key:k.to_string()};
                    match _cmd {
                        Command::Delete {key} => {
                            println!("Executing Delete Operation For : {}", key);
                            start_store.delete(key);
                        }
                        _ => {}
                    }
                } else {
                    println!("You must provide a key value");
                }
            }
            _=>{
                println!("Unknown Command");
            }
        }
    }
}
