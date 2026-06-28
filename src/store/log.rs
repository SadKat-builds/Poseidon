use crc32fast::Hasher;
use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Read , Write};
use std::path::Component;
use std::vec;

use crate::store::log::Command::Put;
use crate::store::memory::Store;

pub enum Command {
    Get { key: String },
    Put { key: String, value: String },
    Delete { key: String },
}
impl Command {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        match self {
            Command::Put { key, value } => {
                buffer.push(0);

                let key_len = key.len() as u32;
                buffer.extend_from_slice(&key_len.to_le_bytes());
                buffer.extend_from_slice(key.as_bytes());

                let val_len = value.len() as u32;
                buffer.extend_from_slice(&val_len.to_le_bytes());
                buffer.extend_from_slice(value.as_bytes());
            }
            Command::Get { key } => {
                buffer.push(1);
                let key_len = key.len() as u32;
                buffer.extend_from_slice(&key_len.to_le_bytes());
                buffer.extend_from_slice(key.as_bytes());
            }
            Command::Delete { key } => {
                buffer.push(2);

                let key_len = key.len() as u32;
                buffer.extend_from_slice(&key_len.to_le_bytes());
                buffer.extend_from_slice(key.as_bytes());
            }
        }
        buffer
    }

    pub fn deserialized(buffer: &[u8]) -> Option<Self> {
        if buffer.is_empty() {
            return None;
        };
        let cmd_type = buffer[0];
        let mut pos: usize = 1;

        match cmd_type {
            1 => {
                let mut key_len_bytes = [0u8; 4];
                key_len_bytes.copy_from_slice(&buffer[pos..pos + 4]);
                let key_len = u32::from_le_bytes(key_len_bytes) as usize;
                pos = pos + 4;

                let key = std::str::from_utf8(&buffer[pos..pos + key_len])
                    .ok()?
                    .to_string();

                Some(Command::Get { key }) 
            }
            2 => {
                 let mut key_len_bytes = [0u8; 4];
                 key_len_bytes.copy_from_slice(&buffer[pos..pos + 4]);
                 let key_len = u32::from_le_bytes(key_len_bytes) as usize;
                 pos = pos + 4;

                 let key = std::str::from_utf8(&buffer[pos..pos + key_len]).ok()?.to_string();

                 Some(Command::Delete { key })
            }
            0 => {
                 
                 // Read key
                 let mut key_len_bytes = [0u8; 4];
                 key_len_bytes.copy_from_slice(&buffer[pos..pos + 4]);
                 let key_len = u32::from_le_bytes(key_len_bytes) as usize;
                 pos += 4;
            
                 let key = std::str::from_utf8(&buffer[pos..pos + key_len]).ok()?.to_string();
                 pos += key_len;
                 // Read value
                 let mut val_len_bytes = [0u8; 4];
                 val_len_bytes.copy_from_slice(&buffer[pos..pos + 4]);
                 let val_len = u32::from_le_bytes(val_len_bytes) as usize;
                 pos += 4;

                 let value = std::str::from_utf8(&buffer[pos..pos + val_len]).ok()?.to_string();
                 
                 Some(Command::Put { key, value })
            }
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Logs {
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

impl Logs {
    pub fn new() -> std::io::Result<Self> {
        std::fs::create_dir_all("storage")?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open("storage/Poseidon.log")?;
        
        let read_file = OpenOptions::new()
            .read(true)
            .open("storage/Poseidon.log")?;
        
        Ok(Self {
            writer: BufWriter::new(file),
            reader: BufReader::new(read_file)
        })
    }

    pub fn append(&mut self, cmd: &Command) -> std::io::Result<()> {
        let clean_cmd = cmd.serialize();

        let mut hasher = Hasher::new();
        hasher.update(&clean_cmd);
        let checksum = hasher.finalize();

        self.writer.write_all(&checksum.to_le_bytes())?;

        let payload_len = clean_cmd.len() as u32;

        let _ = self.writer.write_all(&payload_len.to_le_bytes());

        let _ = self.writer.write_all(&clean_cmd);

        self.writer.flush()?;

        Ok(())
    }

    pub fn replay(&mut self) -> std::io::Result<Store> {
           let mut store = Store::new();
           
           loop {
                
                let mut checksum_bytes = [0u8; 4];
                
                match self.reader.read_exact(&mut checksum_bytes){
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e)
                };
                
                let stored_checksum = u32::from_le_bytes(checksum_bytes);

                let mut payload_len_bytes = [0u8; 4];
                self.reader.read_exact(&mut payload_len_bytes)?;

                let payload_len = u32::from_le_bytes(payload_len_bytes) as usize;

                let mut payload =  vec![0u8; payload_len];
                self.reader.read_exact(&mut payload)?;

                let mut verify_checksum = Hasher::new();
                verify_checksum.update(&payload);
                let computed_checksum = verify_checksum.finalize();

                if stored_checksum != computed_checksum {
                    break;
                }

                if let Some(cmd) = Command::deserialized(&payload){
                    match cmd {
                        Command::Put { key, value } => store.put(key, value),
                        Command::Delete { key } => store.delete(key),
                        _=> {}
                    }
                }                
           }

        Ok(store)
    }
}
