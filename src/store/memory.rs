use std::collections::HashMap;

#[derive(Debug)]
pub struct Store {
       map : HashMap<String, String>
}
impl Store {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }
    pub fn get(&self, key: String) -> Option<&String> {
        self.map.get(&key)  
    }
    pub fn put(&mut self, key: String, value: String) {
           self.map.insert(key,value);
    }
    pub fn delete(&mut self, key : String) {
       self.map.remove(&key);
    }
}