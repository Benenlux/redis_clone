use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Table {
    cache: Arc<RwLock<HashMap<String, String>>>,
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_string() {
        let test_string: Vec<String> = vec!["Currency".to_string(), "Euro".to_string()];
        let mut table = Table::new();
        let result = table.set(test_string[0].clone(), test_string[1].clone());

        assert_eq!(String::from("OK"), result);
    }
}
