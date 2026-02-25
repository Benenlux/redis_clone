use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Table {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    //TODO: handle overwritten keys
    fn set(&mut self, key: String, val: String) -> String {
        match self.cache.write() {
            Ok(mut l_cache) => {
                l_cache.insert(key, val);
                String::from("OK")
            }
            Err(_) => String::from("ERROR"),
        }
    }
    fn get(&self, key: String) -> String {
        match self.cache.read() {
            Ok(l_cache) => {
                let val = l_cache.get(&key);
                match val {
                    Some(val) => val.clone(),
                    None => String::from("(nil)"),
                }
            }
            Err(_) => String::from("ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_STRING: [&str; 2] = ["Currency", "Euro"];
    #[test]
    fn set_string() {
        let mut table = Table::new();
        let result = table.set(String::from(TEST_STRING[0]), String::from(TEST_STRING[1]));

        assert_eq!(String::from("OK"), result);
    }

    #[test]
    fn set_get_string() {
        let mut table = Table::new();
        let set_result = table.set(String::from(TEST_STRING[0]), String::from(TEST_STRING[1]));
        let get_result = table.get(String::from(TEST_STRING[0]));

        assert_eq!(get_result, String::from(TEST_STRING[1]));
        assert_eq!(String::from("OK"), set_result);
    }
}
