use std::collections::HashMap;

#[derive(Debug)]
struct Store {
       map : HashMap<String, String>
}
impl Store {
    fn new() -> Self {
        Store { map: HashMap::new() }
    }
    fn get(&self, key: String) -> Option<&String> {
        self.map.get(&key)  
    }
    fn put(&mut self, key: String, value: String) {
           self.map.insert(key,value);
    }
    fn delete(&mut self, key : String) {
       self.map.remove(&key);
    }
}
enum Command {
    Get {key : String},
    Put {key : String, value : String},
    Delete { key: String}
}

fn main() {

    let mut my_database = Store::new();
    my_database.put(String::from("Rohit"), String::from("Suthar"));
    println!("The value of key Rohit is {:?}", my_database.get(String::from("Rohit")));
}
