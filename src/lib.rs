#![deny(missing_docs)]

//! # kvs
//!
//! `kvs` is a key-value store

use std::collections::HashMap;

/// Key-Value Store
#[derive(Default)]
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// create a KvStore
    pub fn new() -> KvStore {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// retrieve a value from the store
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    /// let key = "key1".to_string();
    /// let value = "value1".to_string();
    /// let mut store = KvStore::new();
    /// store.set(key.to_owned(), value.to_owned());
    /// assert_eq!(Some(value.to_owned()), store.get(key.to_owned()));
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        match self.store.get(&key) {
            Some(v) => Some(v.to_owned()),
            _ => None,
        }
    }

    /// store a value in the store
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    /// let key = "key1".to_string();
    /// let value = "value1".to_string();
    /// let mut store = KvStore::new();
    /// store.set(key.to_owned(), value.to_owned());
    /// assert_eq!(Some(value.to_owned()), store.get(key.to_owned()));
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// remove a value from the store
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    /// let key = "key1".to_string();
    /// let value = "value1".to_string();
    /// let mut store = KvStore::new();
    /// store.set(key.to_owned(), value.to_owned());
    /// assert_eq!(Some(value.to_owned()), store.get(key.to_owned()));
    /// store.remove(key.to_owned());
    /// assert_eq!(None, store.get(key.to_owned()));
    /// ```
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
