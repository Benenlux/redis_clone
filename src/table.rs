use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Table {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[cfg(test)]
mod tests {}
