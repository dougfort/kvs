use crate::{Action, Command, Result};

use failure::format_err;
use std::collections::{BTreeMap};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path};

/// Mutable State
pub struct State {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    file_pointer_map: BTreeMap<String, u64>,
}

impl State {
    pub fn open(dir_path: &Path) -> Result<State> {
        let file_path = dir_path.join("kvs.log");
        let reader = BufReader::new(OpenOptions::new()
            .read(true)
            .open(&file_path)?);
        let writer = BufWriter::new(OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?);
        let file_pointer_map = BTreeMap::new();
            
        let mut state = State {
            reader: reader, 
            writer: writer,
            file_pointer_map,
        };

        state.load_file_pointer_map()?;

        Ok(state)
    }

    fn load_file_pointer_map(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(0))?;
        let mut offset: u64 = 0;

        loop {
            let mut buffer = String::new();
            let count = self.reader.read_line(&mut buffer)?;
            if count == 0 {
                break;
            }
            let cmd: Command = serde_json::from_str(&buffer)?;
            self.file_pointer_map.insert(cmd.key, offset);
            offset += count as u64;
            offset += 1;
        }

        Ok(())
    }

    /// retrieve a value from the store
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.file_pointer_map.get(&key) {
            Some(file_pointer) => {
                self.reader.seek(SeekFrom::Start(*file_pointer))?;
                let mut line = String::new();
                self.reader.read_line(&mut line)?;
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
        let offset = self.writer.seek(SeekFrom::End(0))?;
        self.writer.write_all(s.to_string().as_bytes())?;
        self.writer.write_all("\n".as_bytes())?;
        self.writer.flush()?;
        Ok(offset)
    }
}
