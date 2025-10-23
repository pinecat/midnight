use std::process::exit;

use midnight::Midnight;

use anyhow::Result;

fn main() -> Result<()> {
    let midnight = match Midnight::new() {
        Ok(midnight) => midnight,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    match midnight.enqueue() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
