use crc32fast::Hasher;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};

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
        Ok(Self {
            writer: BufWriter::new(file),
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
}
