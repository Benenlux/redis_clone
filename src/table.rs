use std::collections::HashMap;
use std::sync::RwLock;

use redis_clone::{encode_error, encode_simple_string};

pub struct Table {
    cache: RwLock<HashMap<String, String>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
    // TODO: handle overwritten keys
    pub fn set(&self, key: String, val: String) -> String {
        match self.cache.write() {
            Ok(mut l_cache) => {
                l_cache.insert(key, val);
                encode_simple_string("OK")
            }
            Err(_) => encode_error("Couldn't set key val"),
        }
    }
    pub fn get(&self, key: String) -> String {
        match self.cache.read() {
            Ok(l_cache) => {
                let val = l_cache.get(&key);
                match val {
                    Some(val) => val.clone(),
                    None => encode_simple_string("(nil)"),
                }
            }
            // TODO: handle poisoned locks
            Err(_) => String::from("ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    static TEST_STRING: [&str; 2] = ["Currency", "Euro"];
    #[test]
    fn set_string() {
        let table = Table::new();
        let result = table.set(String::from(TEST_STRING[0]), String::from(TEST_STRING[1]));

        assert_eq!(encode_simple_string("OK"), result);
    }

    #[test]
    fn set_get_string() {
        let table = Table::new();
        let set_result = table.set(String::from(TEST_STRING[0]), String::from(TEST_STRING[1]));
        let get_result = table.get(String::from(TEST_STRING[0]));

        assert_eq!(get_result, String::from(TEST_STRING[1]));
        assert_eq!(encode_simple_string("OK"), set_result);
    }

    #[test]
    fn multithread_set_get() {
        let table = Table::new();

        const NUM_THREADS: usize = 10;
        const INSERTS_PER_THREAD: usize = 100;
        thread::scope(|s| {
            for thread_id in 0..NUM_THREADS {
                let table_ref = &table;

                s.spawn(move || {
                    for i in 0..INSERTS_PER_THREAD {
                        let key = format!("key_{}_{}", thread_id, i);
                        let val = format!("val_{}_{}", thread_id, i);
                        table_ref.set(key, val);
                    }
                });
            }
        });
        let cache = table.cache.read().unwrap();
        let expected_total = NUM_THREADS * INSERTS_PER_THREAD;

        let last_thread_id = NUM_THREADS - 1;
        let key = format!("key_{}_4", last_thread_id);
        let expected_val = format!("val_{}_4", last_thread_id);

        assert_eq!(table.get(key), expected_val);
        assert_eq!(expected_total, cache.len());
    }
}
