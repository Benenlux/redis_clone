use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Table {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_string() {
        let test_string: Vec<String> = vec!["Currency".to_string(), "Euro".to_string()];
        let mut table = Table::new();
        let result = table.set(test_string[0].clone(), test_string[1].clone());

        assert_eq!("OK", result);
    }
}
