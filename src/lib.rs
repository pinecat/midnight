use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

use anyhow::{Context, Error, Result};
use duct::cmd;
use mail_parser::MessageParser;

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

#[derive(Clone, Debug)]
pub struct Midnight {
    /// The raw message (mail)
    raw: String,
    /// The time at which to send the message, specified in a time reconizable to at(1)
    at: String,
}

/// Object to hold raw message and time at which to send said message.
impl Midnight {
    /// Create a new instance of the Midnight object, which holds a raw message string, alongside a
    /// time at which to send the message. Initializes reading mail from STDIN, and also getting
    /// user input from the TTY.
    pub fn new() -> Result<Self> {
        let raw = Midnight::drain_pipe()?;
        let at = Midnight::drain_at()?;
        Ok(Self { raw, at })
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
