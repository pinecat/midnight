use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::OpenOptions;

use anyhow::{Result, anyhow};
use constcat::concat;

use crate::model::Message;
use crate::model::mua::NEOMUTT_XDG_CONFIG_DIR;

/// Default (hardcoded) queue file for midnight
const QUEUE_FILE: &str = concat!(NEOMUTT_XDG_CONFIG_DIR, "/.midnight");

/// Holds a record to a line in the queue file
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Record {
    /// ID of the `at(1)` job
    pub id: String,
    /// Details of the message to be sent
    pub msg: Message,
}

impl Record {
    /// Create a new record (storage for an entry in the queue file) w/ message details and job ID
    pub fn new(id: String, msg: Message) -> Record {
        Record { id, msg }
    }

    /// Append a record to the queue file
    pub fn write(&self) -> Result<()> {
        let writer = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(Self::queue_file()?)?;
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);
        writer.serialize(self)?;
        writer.flush()?;
        Ok(())
    }

    /// Read all records in the queue file
    pub fn read() -> Result<Vec<Self>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(Self::queue_file()?)?;
        let mut records = vec![];
        for record in reader.deserialize() {
            records.push(record?);
        }
        Ok(records)
    }

    /// Find a specific record from an ID
    pub fn find(id: String) -> Result<Self> {
        let rcds = Record::read()?;
        let matching = rcds
            .iter()
            .filter(|rcd| rcd.msg.id == id)
            .collect::<Vec<&Record>>();

        if let Some(rcd) = matching.first() {
            return Ok(rcd.to_owned().to_owned());
        }

        Err(anyhow!(
            "No matching record found in queue file for ID: {}",
            id
        ))
    }

    /// Clear the queue file
    pub fn clear() -> Result<()> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(Self::queue_file()?)?;
        Ok(())
    }

    /// Get the full path of the queue file
    fn queue_file() -> Result<String> {
        Ok(String::from(format!(
            "{}/{}",
            env::var("HOME")?,
            QUEUE_FILE
        )))
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {}\nFrom: {} <{}>\nTo: {} <{}>\nAt: {}",
            self.id,
            self.msg.subject,
            self.msg.sender,
            self.msg.sender_addr,
            self.msg.recipient,
            self.msg.recipient_addr,
            self.msg.at
        )
    }
}
