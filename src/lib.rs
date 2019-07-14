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

pub const COMMAND_MESSAGE: u32 = 1;
pub const ERROR_MESSAGE: u32 = 2;
pub const STRING_MESSAGE: u32 = 3;

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
pub enum Action {
    None,
    Get,
    Set,
    Remove,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub action: Action,
    pub key: String,
    pub value: String,
}

pub enum Message {
    Command(Command),
    Error(String),
    String(String),
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
    
    fn append_command(&mut self, cmd: &Command) -> Result<u64> {
        let s = serde_json::to_string(cmd)?;
        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(s.to_string().as_bytes())?;
        self.file.write_all("\n".as_bytes())?;
        Ok(offset)
    }
}

impl KvsEngine for KvStore {
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
    fn get(&mut self, key: String) -> Result<Option<String>> {
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
    fn set(&mut self, key: String, value: String) -> Result<()> {
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
    fn remove(&mut self, key: String) -> Result<Option<()>> {
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
}

pub fn write_message(writer: &mut std::io::Write, message: &Message) -> Result<()> {
    let (message_type, content) = match message {
        Message::Command(cmd) => (COMMAND_MESSAGE, serde_json::to_string(cmd)?),
        Message::Error(err) => (ERROR_MESSAGE, err.to_string()),
        Message::String(str) => (STRING_MESSAGE, str.to_string()),
    };
    writer.write_all(&message_type.to_be_bytes())?;
    let size = content.len() as u32;
    writer.write_all(&size.to_be_bytes())?;
    writer.write_all(content.as_bytes())?;
    Ok(())
}

pub fn read_message(reader: &mut std::io::Read) -> Result<Message> {
    let mut buffer = [0; std::mem::size_of::<u32>()];
    reader.read_exact(&mut buffer)?;
    let message_type = u32::from_be_bytes(buffer);
    reader.read_exact(&mut buffer)?;
    let message_size = u32::from_be_bytes(buffer);
    let mut content_buffer = String::new();
    let content_size = reader.read_to_string(&mut content_buffer)? as u32;
    if content_size != message_size {
        return Err(format_err!(
            "expected {} bytes got {}",
            message_size,
            content_size
        ));
    }
    match message_type {
        COMMAND_MESSAGE => Ok(Message::Command(serde_json::from_str(&content_buffer)?)),
        ERROR_MESSAGE => Ok(Message::Error(content_buffer)),
        STRING_MESSAGE => Ok(Message::String(content_buffer)),
        _ => Err(format_err!("invalid message type {}", message_type)),
    }
}
