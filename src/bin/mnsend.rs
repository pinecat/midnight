use anyhow::Result;
use midnight::Midnight;

fn main() -> Result<()> {
    Midnight::flags()?;
    Midnight::send()
}
