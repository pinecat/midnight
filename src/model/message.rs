use std::{
    fs,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Error, Result};
use duct::cmd;
use mail_parser::MessageParser;

use crate::model::Mua;

/// Storage for a raw email, and details relevant to midnight
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Message {
    /// The raw, unparsed message (email) String
    #[serde(skip_serializing, skip_deserializing)]
    pub raw: String,
    /// The unique message ID
    pub id: String,
    /// The message's subject
    pub subject: String,
    /// The message's sender
    pub sender: String,
    /// The message's sender's address
    pub sender_addr: String,
    /// The message's recipient
    pub recipient: String,
    /// The message's recipient's address
    pub recipient_addr: String,
    /// Date/time at which the message is scheduled for delivery
    pub at: String,
}

impl Message {
    /// Construct a new [Message] object from a raw email [String] and return it
    pub fn new(raw: String) -> Result<Message> {
        let (id, subject, sender, sender_addr, recipient, recipient_addr) = Message::details(&raw)?;
        let at = String::from("not scheduled");
        Ok(Message {
            raw,
            id,
            subject,
            sender,
            sender_addr,
            recipient,
            recipient_addr,
            at,
        })
    }

    /// Parse details from a raw email [String]
    ///
    /// Specifically, this returns the following values inside a tuple:
    /// - id: The unique message ID
    /// - subject: The message subject (or `[no subject]` if [None])
    /// - sender: The sender's name (or an empty [String] if [None])
    /// - sender_addr: The sender's address
    /// - recipient: The recipient's name (or an empty [String] if [None])
    /// - recipient_addr: The recipient's address
    ///
    /// If the id, sender_addr, or recipient_addr are missing from the message, or if the
    /// [MessageParser] is unable to parse the message as an email, this function will return an
    /// error value.
    fn details(raw: &String) -> Result<(String, String, String, String, String, String)> {
        let rfc5322 = MessageParser::default()
            .parse(raw)
            .context("Unable to parse input as an RFC 5322 MIME message")?;

        // Get the unique ID
        let id = String::from(
            rfc5322
                .message_id()
                .context("There appears to be no unique message ID")?,
        );

        // Get the subject
        let subject = String::from(rfc5322.subject().unwrap_or("[no subject]"));

        // Get the sender
        let sender = rfc5322
            .from()
            .context("From field is missing from RFC 5322 MIME message")?
            .first()
            .context("There appears to be no senders")?;

        // Get the sender's address
        let sender_addr = String::from(sender.address().context("Sender has no address")?);

        // Get the sender's name
        let sender = String::from(sender.name().unwrap_or(""));

        let recipient = rfc5322
            .to()
            .context("To field is missing from RFC 5322 MIME message")?
            .first()
            .context("There appears to be no recipients")?;

        // Get the recipient's address
        let recipient_addr = String::from(recipient.address().context("Recipient has no address")?);

        // Get the recpient's name
        let recipient = String::from(recipient.name().unwrap_or(""));

        // Return all the details
        Ok((id, subject, sender, sender_addr, recipient, recipient_addr))
    }

    pub fn enqueue(&self) -> Result<(String, String)> {
        let echo_cmd = String::from(format!("echo $'{}' | mnsend", self.id));

        let mut args = vec!["-m", "-q", "m"];
        self.at.split(" ").for_each(|arg| args.push(arg));

        let cmd = cmd("at", &args);
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

        let split = job.splitn(4, " ").collect::<Vec<&str>>();
        let jobid = String::from(split[1]);
        let at = String::from(split[3].replace("  ", " "));

        Ok((jobid, at))
    }

    pub fn send(&mut self) -> Result<()> {
        let (rc, account, path) = Mua::search(self)?;
        cmd!("neomutt", "-F", rc, "-F", account, "-H", "-")
            .stdin_bytes(self.raw.as_bytes())
            .run()?;
        fs::remove_file(path)?;
        Ok(())
    }
}
