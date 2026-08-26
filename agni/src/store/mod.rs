mod entry;

pub use entry::Entry;

use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Store {
    data: Arc<DashMap<String, Entry>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        self.data.insert(key.clone(), Entry::new(key, value));
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).map(|e| e.value.clone())
    }

    pub fn delete(&self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn get_as_json(&self, key: &str) -> Option<Result<String, serde_json::Error>> {
        self.data.get(key).map(|e| e.to_json())
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let store = Store::new();
        store.set("name".to_string(), b"agni".to_vec());
        assert_eq!(store.get("name"), Some(b"agni".to_vec()));
    }

    #[test]
    fn test_get_missing_key() {
        let store = Store::new();
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn test_overwrite_value() {
        let store = Store::new();
        store.set("key".to_string(), b"first".to_vec());
        store.set("key".to_string(), b"second".to_vec());
        assert_eq!(store.get("key"), Some(b"second".to_vec()));
    }

    #[test]
    fn test_delete_existing_key() {
        let store = Store::new();
        store.set("key".to_string(), b"value".to_vec());
        assert!(store.delete("key"));
        assert_eq!(store.get("key"), None);
    }

    #[test]
    fn test_delete_missing_key() {
        let store = Store::new();
        assert!(!store.delete("missing"));
    }

    #[test]
    fn test_shared_across_clones() {
        let store = Store::new();
        let store2 = store.clone();
        store.set("key".to_string(), b"value".to_vec());
        assert_eq!(store2.get("key"), Some(b"value".to_vec()));
    }

    #[test]
    fn test_get_as_json_contains_key_and_base64_value() {
        let store = Store::new();
        store.set("hello".to_string(), b"world".to_vec());
        let json = store.get_as_json("hello").unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["key"], "hello");
        assert_eq!(parsed["value"], "d29ybGQ=");
        assert!(parsed["id"].is_string());
    }

    #[test]
    fn test_get_as_json_missing_key() {
        let store = Store::new();
        assert!(store.get_as_json("missing").is_none());
    }
}
