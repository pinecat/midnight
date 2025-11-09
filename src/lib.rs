use std::io::{BufRead, BufReader};
use std::process::exit;

use anyhow::Result;
use duct::cmd;
use flager::{Flag, Parser, new_flag};

mod model;
pub mod quirk;
pub use model::Message;
use model::Pipe;
pub use model::Record;

/// Program entry point
pub struct Midnight;

impl Midnight {
    /// Parse CLI flags
    pub fn flags() -> Result<()> {
        let parser = Parser::new();

        let help_msg = "Print help message then quit";
        let queue_msg = "List items in queue (alias for 'mnq')";
        let remove_msg = "Remove item from queue via job ID (alias for 'mnrm')";
        let send_msg = "Send message in queue via message ID (alias for 'mnsend')";
        let version_msg = "Print the version then quit";

        let help_flag: Flag<bool> = new_flag!("-h", "--help").help(help_msg);
        let queue_flag: Flag<bool> = new_flag!("-q", "--queue").help(queue_msg);
        let remove_flag: Flag<String> = new_flag!("-r", "--remove").help(remove_msg);
        let send_flag: Flag<bool> = new_flag!("-s", "--send").help(send_msg);
        let version_flag: Flag<bool> = new_flag!("-V", "--version").help(version_msg);

        if parser.parse(&help_flag).unwrap() == true {
            println!("midnight: Send mail later via batch queueing");
            println!();
            println!("Usage: <message> | midnight");
            println!("Usage: <message> | mn");
            println!("Usage: mnq");
            println!("Usage: mnrm <jobid>");
            println!("Usage: <message_id> | mnsend");
            println!();
            println!("Options:");
            println!("\t-h, --help\t{}", help_msg);
            println!("\t-q, --queue\t{}", queue_msg);
            println!("\t-r, --remove\t{}", remove_msg);
            println!("\t-s, --send\t{}", send_msg);
            println!("\t-V, --version\t{}", version_msg);
            exit(0);
        }

        if parser.parse(&queue_flag).unwrap() == true {
            Self::list()?;
            exit(0);
        }

        if let Some(jobid) = parser.parse(&remove_flag) {
            if jobid.is_empty() {
                eprintln!("Usage: midnight -r <jobid>");
                exit(1);
            }
            Self::rm(&jobid)?;
            exit(0);
        }

        if parser.parse(&send_flag).unwrap() == true {
            Self::send()?;
            exit(0);
        }

        if parser.parse(&version_flag).unwrap() == true {
            println!("midnight v{}", env!("CARGO_PKG_VERSION"));
            exit(0);
        }

        Ok(())
    }

    /// Schedule a message to be sent later
    pub fn enqueue() -> Result<()> {
        // Get raw message and parse it
        let raw = Pipe::drain()?;
        let mut msg = Message::new(raw)?;

        // Figure out what time the user wants to schedule the delivery
        msg.at = Pipe::user(Some("What time? "))?;

        // Queue message for delivery via at(1)
        let (jobid, at) = msg.enqueue()?;
        msg.at = at;

        // Record message details in the queue file
        let rcd = Record::new(jobid, msg);
        rcd.write()?;

        Ok(())
    }

    /// Send a message that was scheduled for delivery
    pub fn send() -> Result<()> {
        let id = Pipe::drain()?;
        let mut rcd = Record::find(id)?;
        rcd.msg.send()
    }

    /// Remove a message from the queue that was scheduled, but hasn't been sent yet
    pub fn rm(jobid: &str) -> Result<()> {
        cmd!("atrm", jobid).run()?;
        Self::clean()?;
        Ok(())
    }

    /// List scheduled messages in the queue
    pub fn list() -> Result<()> {
        Self::clean()?;
        let rcds = Record::read()?;
        for (i, rcd) in rcds.iter().enumerate() {
            println!("{}", rcd);
            if i < rcds.len() - 1 {
                println!();
            }
        }
        Ok(())
    }

    /// Remove stale entries from the queue file
    pub fn clean() -> Result<()> {
        // Read in jobs from atq(1)
        let cmd = cmd!("atq");
        let reader = cmd.stderr_to_stdout().reader()?; // {

        let mut jobs = vec![];
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            let line = line?;
            let jobid = String::from(line.split_whitespace().collect::<Vec<&str>>()[0]);
            jobs.push(jobid);
        }

        // Read in records from the queue files
        let rcds = Record::read()?;

        // Find records that still have a job in atq(1)
        let scheduled = rcds
            .iter()
            .filter(|rcd| jobs.contains(&rcd.id))
            .collect::<Vec<&Record>>();

        // Clear the queue file
        Record::clear()?;

        // Write records back out that are still in atq(1)
        for rcd in scheduled {
            rcd.write()?;
        }

        Ok(())
    }
}
