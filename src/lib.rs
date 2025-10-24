use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

use anyhow::{Context, Error, Result};
use duct::cmd;
use mail_parser::MessageParser;

const NEOMUTT_XDG_CONFIG_DIR: &str = ".config/neomutt";

#[derive(Clone, Debug)]
pub struct Midnight {
    raw: String,
    at: String,
}

impl Midnight {
    pub fn new() -> Result<Self> {
        let raw = Midnight::drain_pipe()?;
        let at = Midnight::drain_at()?;

        Ok(Self { raw, at })
    }

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

    fn drain_at() -> Result<String> {
        let mut buffer = String::new();
        let stdin = OpenOptions::new().read(true).open("/dev/tty")?;
        let mut reader = BufReader::new(stdin);
        print!("What time? ");
        io::stdout().flush()?;
        reader.read_line(&mut buffer)?;
        if let Some('\n') = buffer.chars().next_back() {
            buffer.pop();
        }
        Ok(buffer)
    }

    fn authenticate(&self) -> Result<String> {
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

    pub fn forkauth(&self) -> Result<(String, String)> {
        let address = self.authenticate()?;
        let path = format!("{}/{}", env!("HOME"), NEOMUTT_XDG_CONFIG_DIR);
        let rc = format!("{}/neomuttrc", path);
        let account = String::from(cmd!("rg", "-l", "-g", "!tmp", &address, &path).read()?);
        Ok((rc, account))
    }

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
