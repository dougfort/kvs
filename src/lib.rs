//! # kvs
//!
//! `kvs` is a key-value store
//!

use failure::{format_err, Error};
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use std::result;
use std::sync::{Arc, Mutex};

pub const COMMAND_MESSAGE: u32 = 1;
pub const ERROR_MESSAGE: u32 = 2;
pub const STRING_MESSAGE: u32 = 3;

pub mod state;
pub mod thread_pool;

/// alias for Result<>
pub type Result<T> = result::Result<T, Error>;

/// Key-Value Store
pub struct KvStore {
    state: Arc<Mutex<Box<state::State>>>,
}

pub trait KvsEngine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<Option<()>>;
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
        let state = state::State::open(dir_path)?;
        let store = KvStore {
            state: Arc::new(Mutex::new(Box::new(state))),
        };

        Ok(store)
    }
}

impl Clone for KvStore {
    fn clone(&self) -> KvStore {
        KvStore {
            state: Arc::clone(&self.state),
        }
    }
}

impl KvsEngine for KvStore {
    /// retrieve a value from the store
    fn get(&self, key: String) -> Result<Option<String>> {
        // We unwrap() the return value to assert that we are not expecting
        // threads to ever fail while holding the lock.
        let mut state = self.state.lock().unwrap();
        state.get(key)
    }

    /// store a value in the store
    fn set(&self, key: String, value: String) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.set(key, value)
    }

    /// remove a value from the store
    fn remove(&self, key: String) -> Result<Option<()>> {
        let mut state = self.state.lock().unwrap();
        state.remove(key)
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
    let mut buffer: [u8; std::mem::size_of::<u32>()] = [0; std::mem::size_of::<u32>()];
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

#[test]
fn test_get() -> Result<()> {
    let store1 = KvStore::open(Path::new(""))?;
    store1.set("key1".to_owned(), "value1".to_owned())?;
    let store2 = store1.clone();
    let val = store2.get("key1".to_owned())?;
    assert!(val == Some("value1".to_owned()));
    Ok(())
}
