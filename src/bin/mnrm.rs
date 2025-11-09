use anyhow::Result;
use midnight::Midnight;

fn main() -> Result<()> {
    Midnight::flags()?;
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: mnrm <jobid>");
        std::process::exit(1);
    }
    Midnight::rm(&args[1])
}
