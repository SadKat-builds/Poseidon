use poseidon::store::{log::Logs, memory::Store, log::Command};
use std::io::{self, Write};


fn main() {
    let mut start_store = Store::new();
    let mut _init_logs = Logs::new().unwrap();
    loop {
        println!("Welcome to Poseidon! ");
        println!("List of Operations Available : GET | PUT | DELETE | EXIT");
        
        print!("Enter your command : ");
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
            Some("GET") => {
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
            Some("PUT") => {
                if let (Some(k),Some(v)) = (key,value) {
                    let cmd = Command::Put{key:k.to_string(), value:v.to_string()};
                    match cmd {
                        Command::Put {ref key , ref value} => {
                            println!("Executing Put Operation For Key : {} , Value : {}", key , value);
                            _init_logs.append(&cmd).unwrap();
                            start_store.put(key.to_string(), value.to_string());
                        }
                        _ => {}
                    }
                } else {
                    println!("You must provide a key value and a value");
                }
            }
            Some("DELETE") => {
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
