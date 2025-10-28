use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

use anyhow::{Context, Error, Result, anyhow};
use duct::cmd;
use mail_parser::MessageParser;
use regex::Regex;

/// Defualt (hardcoded) config dir for neomutt
const NEOMUTT_XDG_CONFIG_DIR: &str = ".config/neomutt";

/// Trait to implement chomp (removes newline from end of a [String], if there is one). Inspired by
/// Ruby's chomp. If it's edible, you can chomp it!
trait Edible {
    fn chomp(&mut self);
}

/// Simple implementation for chomp on a [String].
impl Edible for String {
    fn chomp(&mut self) {
        if let Some('\n') = self.chars().next_back() {
            self.pop();
        }
    }
}

/// Object to hold raw message, the messages' unique ID, and time at which to send said message.
#[derive(Clone, Debug)]
pub struct Midnight {
    /// The raw message (mail)
    raw: String,
    /// Unique message ID
    id: String,
    /// The time at which to send the message, specified in a time reconizable to at(1)
    at: String,
}

/// Implement functions for Midnight.
impl Midnight {
    /// Create a new instance of the Midnight object, which holds a raw message string, alongside
    /// the unique messasge ID, and a time at which to send the message. Initializes reading mail
    /// from STDIN, and also getting user input from the TTY.
    pub fn new() -> Result<Self> {
        let raw = Midnight::drain_pipe()?;
        let id = Midnight::id(&raw)?;
        let at = Midnight::drain_at()?;
        Ok(Self { raw, id, at })
    }

    /// Read in piped input from STDIN (in this case, the raw mail message).
    fn drain_pipe() -> Result<String> {
        let mut buffer = String::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            buffer.push_str(&format!("{line}\n"));
        }
        let buffer = String::from(buffer.trim());
        Ok(buffer)
    }

    /// Read input from user in a TTY (need to specifically use /dev/tty since we are using STDIN
    /// for piped input).
    fn drain_at() -> Result<String> {
        let mut buffer = String::new();
        let stdin = OpenOptions::new().read(true).open("/dev/tty")?;
        let mut reader = BufReader::new(stdin);
        print!("What time? ");
        io::stdout().flush()?;
        reader.read_line(&mut buffer)?;
        buffer.chomp();
        Ok(buffer)
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
    pub fn maildir(&self) -> Result<Vec<String>> {
        let re = Regex::new(r#"set (folder|postponed) = "([^"]*)""#)?;
        let (_, account_file) = self.forkauth()?;
        let file = File::open(account_file)?;
        let reader = BufReader::new(file);
        let mut map: HashMap<String, String> = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = re.captures(&line) {
                map.insert(caps[1].to_string(), caps[2].to_string());
            }
        }

        let folder = match map.get("folder") {
            Some(folder) => folder.replace(|c: char| c == '~', env!("HOME")),
            None => {
                return Err(anyhow!(
                    "You must specify the top-level maildir in your account file"
                ));
            }
        };

        let postponed = match map.get("postponed") {
            Some(postponed) => postponed.replace(|c: char| !c.is_alphanumeric(), ""),
            None => {
                return Err(anyhow!(
                    "You must specify the postponed maildir in your account file"
                ));
            }
        };

        Ok(vec![
            String::from(format!("{}/{}/new", folder, postponed)),
            String::from(format!("{}/{}/cur", folder, postponed)),
            String::from(format!("{}/{}/tmp", folder, postponed)),
        ])
    }

    /// Search for a draft, matching on the unique message ID stored in the object. Gets search
    /// paths from the [Midnight::maildir] function. Function returns the full path of the draft on
    /// disk, if a match is found.
    pub fn search_drafts(&self) -> Result<String> {
        let folders = self.maildir()?;

        let mut path = String::new();
        for folder in folders {
            let drafts = fs::read_dir(folder)?;
            for draft in drafts {
                let draft = draft?;
                let raw = fs::read_to_string(draft.path())?;
                let msg = match MessageParser::default().parse(&raw) {
                    Some(msg) => msg,
                    None => continue,
                };
                let id = match msg.message_id() {
                    Some(id) => id.to_owned(),
                    None => continue,
                };

                if id == self.id {
                    path = draft.path().to_string_lossy().to_string();
                }
            }
        }

        if path.is_empty() {
            return Err(anyhow!("Could not get path of message on disk"));
        }

        Ok(path)
    }

    /// Parse the unique message ID from a raw String.
    pub fn id(raw: &String) -> Result<String> {
        let message = MessageParser::default()
            .parse(raw)
            .context("Unable to parse input as an RFC 5322 MIME message")?;

        match message.message_id() {
            Some(id) => Ok(id.to_owned()),
            _ => Err(anyhow!("Not message ID was found")),
        }
    }

    /// Parse the raw message to retrieve the sender (user might have multiple accounts).
    pub fn sender(&self) -> Result<String> {
        // Parse the message
        let message = MessageParser::default()
            .parse(&self.raw)
            .context("Unable to parse input as an RFC 5322 MIME message")?;

        // Get the sender
        let sender = message
            .from()
            .context("From field is missing from RFC 5322 MIME message")?
            .first()
            .context("There appears to be no senders")?;

        // Get the sender's address
        let address = sender
            .address()
            .context("Sender has no address")?
            .to_owned();

        Ok(address)
    }

    /// Figure out what file(s) to source based off the sender. See [Midnight::sender] for
    /// more details. Right now, this assumes that you have a different file for each account you
    /// use in neomutt.
    pub fn forkauth(&self) -> Result<(String, String)> {
        let address = self.sender()?;
        let path = format!("{}/{}", env!("HOME"), NEOMUTT_XDG_CONFIG_DIR);
        let rc = format!("{}/neomuttrc", path);
        let account = String::from(cmd!("rg", "-l", "-g", "!tmp", &address, &path).read()?);
        Ok((rc, account))
    }

    /// Escape any single quotes (') for the echo command.
    fn escape(&self) -> String {
        let mut indicies = Vec::new();
        let mut count = 0;
        for (i, c) in self.raw.chars().enumerate() {
            if c == '\'' {
                indicies.push(i + count);
                count += 1;
            }
        }

        let mut buffer = String::from(&self.raw);
        for i in indicies {
            buffer.insert(i, '\\');
        }

        buffer
    }

    /// Queue a mail to send at a later time.
    pub fn enqueue(&self) -> Result<()> {
        let (rc, account) = self.forkauth()?;
        let echo_cmd = String::from(format!(
            "echo $'{}' | neomutt -F {} -F {} -H -",
            self.escape(),
            rc,
            account
        ));

        let cmd = cmd!("at", "-m", "-q", "m", &self.at);
        let reader = cmd.stdin_bytes(echo_cmd).stderr_to_stdout().reader()?; // {

        let reader = BufReader::new(reader);
        let job = match reader.lines().last() {
            Some(job) => job?,
            None => {
                return Err(Error::msg(
                    "Invalid time, could not schedule mail for delivery",
                ));
            }
        };

        println!("{}", job.split(" ").collect::<Vec<&str>>()[1]);

        Ok(())
    }
}
