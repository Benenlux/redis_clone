use crate::utils::{AsResp, RespReplies, encode_error, encode_simple_string};
use std::collections::HashMap;
use std::io::{IoSlice, Write};
use std::sync::RwLock;

pub struct Table {
    cache: RwLock<HashMap<String, String>>,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
    pub fn set<W: Write>(&self, key: String, val: String, writer: &mut W) -> std::io::Result<()> {
        let response: &[u8] = match self.cache.write() {
            Ok(mut l_cache) => {
                l_cache.insert(key, val);
                RespReplies::OKString.to_resp()
            }
            Err(_) => RespReplies::NullString.to_resp(),
        };
        writer.write_all(response)
    }

    pub fn get(&self, key: &String) -> String {
        match self.cache.read() {
            Ok(l_cache) => {
                let val = l_cache.get(key);
                match val {
                    Some(val) => encode_simple_string(val.clone()),
                    None => "$-1\r\n".to_string(),
                }
            }
            // TODO: handle poisoned locks
            Err(_) => encode_error("Poisoned lock"),
        }
    }

    // Retrieves a value from the cache by key and writes its RESP representation
    /// to the given writer.
    ///
    /// If the key is missing or the cache lock is poisoned, a null RESP reply is written instead.
    pub fn get_write<W: Write>(&self, key: &str, writer: &mut W) -> std::io::Result<()> {
        match self.cache.read() {
            Ok(l_cache) => match l_cache.get(key) {
                Some(val) => {
                    val.as_resp().write_to(writer)?;
                    Ok(())
                }
                None => {
                    drop(l_cache);
                    writer.write_all(RespReplies::NullString.to_resp())
                }
            },
            // TODO: handle poisoned locks
            Err(_) => writer.write_all(RespReplies::NullString.to_resp()),
        }
    }

    pub fn get_with<F, R>(&self, key: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(Option<&str>) -> R,
    {
        match self.cache.read() {
            Ok(guard) => Ok(f(guard.get(key).map(|s| s.as_str()))),
            Err(_) => Err(encode_error("Poisoned lock")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::sink;
    use std::thread;

    use super::*;

    static TEST_STRING: [&str; 2] = ["Currency", "Euro"];

    #[test]
    fn set_string() {
        let table = Table::new();
        let mut buf = Vec::new();

        table
            .set(
                String::from(TEST_STRING[0]),
                String::from(TEST_STRING[1]),
                &mut buf,
            )
            .unwrap();

        assert_eq!(buf, RespReplies::OKString.to_resp());
    }

    #[test]
    fn set_and_get_write_existing_key() {
        let table = Table::new();
        let mut set_buf = Vec::new();
        let mut get_buf = Vec::new();

        table
            .set(
                String::from(TEST_STRING[0]),
                String::from(TEST_STRING[1]),
                &mut set_buf,
            )
            .unwrap();
        assert_eq!(set_buf, RespReplies::OKString.to_resp());

        table.get_write(TEST_STRING[0], &mut get_buf).unwrap();

        // Compare against whatever `val.as_resp()` writes for TEST_STRING[1]
        let mut expected_buf = Vec::new();
        TEST_STRING[1]
            .to_string()
            .as_resp()
            .write_to(&mut expected_buf)
            .unwrap();
        assert_eq!(get_buf, expected_buf);
    }

    #[test]
    fn get_write_missing_key() {
        let table = Table::new();
        let mut buf = Vec::new();

        table.get_write("nonexistent_key", &mut buf).unwrap();

        assert_eq!(buf, RespReplies::NullString.to_resp());
    }

    #[test]
    fn overwrite_existing_key() {
        let table = Table::new();
        let mut buf = Vec::new();

        table
            .set("key".to_string(), "initial".to_string(), &mut sink())
            .unwrap();
        table
            .set("key".to_string(), "updated".to_string(), &mut sink())
            .unwrap();

        table.get_write("key", &mut buf).unwrap();

        let mut expected_buf = Vec::new();
        "updated"
            .to_string()
            .as_resp()
            .write_to(&mut expected_buf)
            .unwrap();
        assert_eq!(buf, expected_buf);
    }

    #[test]
    fn get_with_closure() {
        let table = Table::new();
        table
            .set(
                String::from(TEST_STRING[0]),
                String::from(TEST_STRING[1]),
                &mut sink(),
            )
            .unwrap();

        // Check existing key
        let found_len = table
            .get_with(TEST_STRING[0], |val| val.map(|s| s.len()))
            .unwrap();
        assert_eq!(found_len, Some(TEST_STRING[1].len()));

        // Check missing key
        let missing = table.get_with("missing", |val| val.is_none()).unwrap();
        assert!(missing);
    }

    #[test]
    fn multithread_set_and_concurrent_get() {
        let table = Table::new();

        const NUM_THREADS: usize = 10;
        const INSERTS_PER_THREAD: usize = 100;

        // 1. Concurrent writes across 10 threads
        thread::scope(|s| {
            for thread_id in 0..NUM_THREADS {
                let table_ref = &table;
                s.spawn(move || {
                    for i in 0..INSERTS_PER_THREAD {
                        let key = format!("key_{}_{}", thread_id, i);
                        let val = format!("val_{}_{}", thread_id, i);
                        table_ref.set(key, val, &mut sink()).unwrap();
                    }
                });
            }
        });

        let expected_total = NUM_THREADS * INSERTS_PER_THREAD;
        assert_eq!(expected_total, table.cache.read().unwrap().len());

        // 2. Concurrent reads and writes simultaneously to stress-test the RwLock
        thread::scope(|s| {
            // Reader threads verifying inserted keys
            for thread_id in 0..NUM_THREADS {
                let table_ref = &table;
                s.spawn(move || {
                    let key = format!("key_{}_4", thread_id);
                    let expected_val = format!("val_{}_4", thread_id);

                    let mut buf = Vec::new();
                    table_ref.get_write(&key, &mut buf).unwrap();

                    let mut expected_buf = Vec::new();
                    expected_val.as_resp().write_to(&mut expected_buf).unwrap();
                    assert_eq!(buf, expected_buf);
                });
            }

            // Concurrent writer updating a separate key while readers are active
            let table_ref = &table;
            s.spawn(move || {
                table_ref
                    .set(
                        "concurrent_key".to_string(),
                        "value".to_string(),
                        &mut sink(),
                    )
                    .unwrap();
            });
        });

        assert_eq!(expected_total + 1, table.cache.read().unwrap().len());
    }
}
