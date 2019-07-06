//! # kvs
//!
//! `kvs` is a key-value store
//!

use failure::{format_err, Error};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{BufReader, Seek, Write};
use std::path::Path;
use std::result;

/// alias for Result<>
pub type Result<T> = result::Result<T, Error>;

/// Key-Value Store
pub struct KvStore {
    file: File,
    file_pointer_map: HashMap<String, u64>,
}

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<Option<()>>;
}

#[derive(Serialize, Deserialize, Debug)]
enum Action {
    None,
    Set,
    Remove,
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    action: Action,
    key: String,
    value: String,
}

impl KvStore {
    /// create a KvStore
    pub fn open(dir_path: &Path) -> Result<KvStore> {
        let file_path = dir_path.join("kvs.log");
        let mut store = KvStore {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path)?,
            file_pointer_map: HashMap::new(),
        };

        store.file.seek(SeekFrom::Start(0))?;
        let reader = BufReader::new(&store.file);
        let mut offset: u64 = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            let cmd: Command = serde_json::from_str(&line)?;
            store.file_pointer_map.insert(cmd.key, offset);
            offset += line.len() as u64;
            offset += 1;
        }

        Ok(store)
    }

    /// retrieve a value from the store
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path;
    /// use kvs::KvStore;
    ///
    /// let key = "key1".to_string();
    /// let value = "value1".to_string();
    /// let path = Path::new("kvs.log");
    ///
    /// let mut store = KvStore::open(path)?;
    /// store.set(key.to_owned(), value.to_owned());
    /// assert_eq!(Some(value.to_owned()), store.get(key.to_owned()));
    /// ```
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.file_pointer_map.get(&key) {
            Some(file_pointer) => {
                self.file.seek(SeekFrom::Start(*file_pointer))?;
                let mut reader = BufReader::new(&self.file);
                let mut line = String::new();
                reader.read_line(&mut line)?;
                let cmd: Command = serde_json::from_str(&line)?;
                match cmd.action {
                    Action::Set => Ok(Some(cmd.value)),
                    _ => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// store a value in the store
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path;
    /// use kvs::KvStore;
    ///
    /// let key = "key1".to_string();
    /// let value = "value1".to_string();
    /// let path = Path::new("kvs.log");
    ///
    /// let mut store = KvStore::open()?;
    /// store.set(key.to_owned(), value.to_owned());
    /// assert_eq!(Some(value.to_owned()), store.get(key.to_owned()));
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command {
            action: Action::Set,
            key,
            value,
        };
        let offset = self.append_command(&cmd)?;
        self.file_pointer_map.insert(cmd.key, offset);

        Ok(())
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
    pub fn remove(&mut self, key: String) -> Result<Option<()>> {
        let get_result = self.get(key.to_owned())?;
        if get_result.is_none() {
            return Err(format_err!("Key not found"));
        };
        let cmd = Command {
            action: Action::Remove,
            key,
            value: String::new(),
        };
        let offset = self.append_command(&cmd)?;
        self.file_pointer_map.insert(cmd.key, offset);
        Ok(Some(()))
    }

    fn append_command(&mut self, cmd: &Command) -> Result<u64> {
        let s = serde_json::to_string(cmd)?;
        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(s.to_string().as_bytes())?;
        self.file.write_all("\n".as_bytes())?;
        Ok(offset)
    }
}
