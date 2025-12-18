use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Write},
};

use anyhow::Result;

use quirks::Edible;

/// Work with input from $stdin
pub struct Pipe;

impl Pipe {
    /// Drain $stdin when output is piped to this program
    pub fn drain() -> Result<String> {
        let mut buffer = String::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            buffer.push_str(&format!("{line}\n"));
        }
        let buffer = String::from(buffer.trim());
        Ok(buffer)
    }

    /// Get user input with an optional prompt
    ///
    /// We have to use this function to get user input from $stdin, since [Pipe::drain] takes over
    /// $stdin if there is input waiting to be piped.
    pub fn user(prompt: Option<&str>) -> Result<String> {
        let mut buffer = String::new();
        let stdin = OpenOptions::new().read(true).open("/dev/tty")?;
        let mut reader = BufReader::new(stdin);
        match prompt {
            Some(prompt) => print!("{}", prompt),
            None => {}
        }
        io::stdout().flush()?;
        reader.read_line(&mut buffer)?;
        Ok(buffer.chomp())
    }
}
