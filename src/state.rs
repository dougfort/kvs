use crate::{Action, Command, Result};

use failure::format_err;
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Mutable State
pub struct State {
    file: File,
    file_path: PathBuf,
    file_pointer_map: BTreeMap<String, u64>,
}

impl State {
    pub fn open(dir_path: &Path) -> Result<State> {
        let file_path = dir_path.join("kvs.log");
        let mut state = State {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&file_path)?,
            file_path: file_path,
            file_pointer_map: BTreeMap::new(),
        };

        state.file.seek(SeekFrom::Start(0))?;
        let reader = BufReader::new(&state.file);
        let mut offset: u64 = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            let cmd: Command = serde_json::from_str(&line)?;
            state.file_pointer_map.insert(cmd.key, offset);
            offset += line.len() as u64;
            offset += 1;
        }

        Ok(state)
    }

    /// retrieve a value from the store
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
        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(s.to_string().as_bytes())?;
        self.file.write_all("\n".as_bytes())?;
        Ok(offset)
    }
}
