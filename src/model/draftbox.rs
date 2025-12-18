use std::{env, fs};

use anyhow::Result;
use constcat::concat;
use csv::Trim;
use quirks::Odyssey;

use crate::model::mua::NEOMUTT_XDG_CONFIG_DIR;

/// Default (hardcoded) config file path for neomutt
const DRAFTBOX_FILE: &str = concat!(NEOMUTT_XDG_CONFIG_DIR, "/.draftboxes");

/// Storage for accounts and corresponding draftboxes
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Draftbox {
    /// Email address
    pub address: String,
    /// Path to draftbox associated with the email address
    pub path: String,
}

impl Draftbox {
    /// Create a new [Draftbox] object
    pub fn new(address: &str, path: &str) -> Self {
        let address = String::from(address);
        let path = String::from(path);
        Draftbox { address, path }
    }

    /// Check if there is a draftboxces file
    pub fn exists() -> bool {
        match fs::exists(DRAFTBOX_FILE) {
            Ok(res) if res == true => true,
            Ok(_) => false,
            Err(_) => false,
        }
    }

    /// Load a list of draftboxes specified by the user
    pub fn load() -> Result<Vec<Self>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'=')
            .trim(Trim::All)
            .comment(Some(b'#'))
            .from_path(Self::draftboxes_file()?)?;
        let mut draftboxes = vec![];
        for draftbox in reader.deserialize::<Draftbox>() {
            let mut draftbox = draftbox?;
            draftbox.path = draftbox.path.expand()?;
            draftboxes.push(draftbox);
        }
        Ok(draftboxes)
    }

    /// Return maildir folders on draftbox directory
    pub fn folders(&self) -> Vec<String> {
        vec![
            String::from(format!("{}/new", self.path)),
            String::from(format!("{}/cur", self.path)),
            String::from(format!("{}/tmp", self.path)),
        ]
    }

    /// Get the full path of the draftbox file
    fn draftboxes_file() -> Result<String> {
        Ok(String::from(format!(
            "{}/{}",
            env::var("HOME")?,
            DRAFTBOX_FILE
        )))
    }
}
