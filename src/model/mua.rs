use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use anyhow::{Result, anyhow};
use constcat::concat;
use duct::cmd;
use mail_parser::MessageParser;
use regex::Regex;

use crate::model::Message;
use crate::model::Draftbox;

macro_rules! nop { () => { {}  }; }

/// Default (hardcoded) config dir for neomutt
pub const NEOMUTT_XDG_CONFIG_DIR: &str = ".config/neomutt";

/// Default (hardcoded) config file path for neomutt
const RC: &str = concat!(NEOMUTT_XDG_CONFIG_DIR, "/neomuttrc");

/// Functions for interacting with the mail user agent (neomutt)
pub struct Mua;

impl Mua {
    /// Search for message to be sent to the appropriate draftbox
    ///
    /// Returns location to the default RC file for neomutt (currently hardcoded), location of the
    /// account file to use for sending, and the path of the draft on disk. Also populates the
    /// `raw` file inside of the [Message].
    pub fn search(msg: &mut Message) -> Result<(String, String, String)> {
        let (rc, account) = Self::account(&msg.sender_addr)?;
        let maildirs = Self::maildirs(&account, &msg.sender_addr)?;
        let (path, raw) = Self::draft(maildirs, &msg.id)?;
        msg.raw = raw;
        Ok((rc, account, path))
    }

    pub fn account(from: &String) -> Result<(String, String)> {
        let path = format!("{}/{}", env::var("HOME")?, NEOMUTT_XDG_CONFIG_DIR);
        let rc = String::from(format!("{}/{}", env::var("HOME")?, RC));
        let account =
            String::from(cmd!("rg", "-l", "-g", "!tmp", "-g", "!signatures", from, &path).read()?);
        Ok((rc, account))
    }

    /// Uses [Midnight::forkauth] to get the account file, then searches that file for the
    /// following lines:
    ///
    /// ```
    /// set folder = "<path-to-root-level-maildir>"
    /// set postponed = "<relative-path-to-draftbox>"
    /// ```
    ///
    /// It is not difficult to see there are some assumptions being made here, which may not work
    /// for all neomutt configs. First of all, it assumes that the config has a separate file for
    /// each account that the user accesses with neomutt. Second, it assumes that the user is
    /// settings both the `folder` and `postponed` options within that same account file.
    ///
    /// If all those assumptions fall into place, however, the function will grab the
    /// root-level-maildir (folder) and concat it with the maildir folder (new, cur, tmp) inside
    /// the draftbox (postponed) maildir, and return those three paths.
    fn maildirs(account: &String, from: &String) -> Result<Draftbox> {
        if Draftbox::exists() {
            let draftboxes = Draftbox::load()?;
            match draftboxes.iter().find(|d| d.address == *from) {
                Some(draftbox) => return Ok(draftbox.to_owned()),
                None => nop!(),
            }
        }

        let re = Regex::new(r#"set (folder|postponed) = "([^"]*)""#)?;
        let file = File::open(account)?;
        let reader = BufReader::new(file);
        let mut map: HashMap<String, String> = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = re.captures(&line) {
                map.insert(caps[1].to_string(), caps[2].to_string());
            }
        }

        let folder = map
            .get("folder")
            .expect("You must specify the top-level maildir in your account file")
            .replace(|c: char| c == '~', &env::var("HOME")?);

        let postponed = map
            .get("postponed")
            .expect("You must specify the postponed maildir in your account file")
            .replace(|c: char| !c.is_alphanumeric(), "");

        Ok(Draftbox::new(from, &format!("{}/{}", folder, postponed)))
    }

    /// Search for a draft, matching on the unique message ID stored in the object
    ///
    /// Gets search paths from the [Self::maildirs] function. Function returns the full path of the
    /// draft on disk, alongside the contents of the message, if a match is found.
    fn draft(draftbox: Draftbox, id: &String) -> Result<(String, String)> {
        let mut path = String::new();
        let mut raw = String::new();
        for maildir in draftbox.folders() {
            let drafts = fs::read_dir(maildir)?;
            for draft in drafts {
                let draft = draft?;
                let msg = fs::read_to_string(draft.path())?;
                let msg = match MessageParser::default().parse(&msg) {
                    Some(msg) => msg,
                    None => continue,
                };
                let draftid = match msg.message_id() {
                    Some(id) => id.to_owned(),
                    None => continue,
                };

                if draftid == *id {
                    path = draft.path().to_string_lossy().to_string();
                    raw = fs::read_to_string(&path)?;
                }
            }
        }

        if path.is_empty() {
            return Err(anyhow!("Could not get path of message on disk"));
        }

        Ok((path, raw))
    }
}
