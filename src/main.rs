mod config;
mod terminal;

use crate::terminal::TerminalGuard;
use anyhow::{Context, Result};
use std::io;
use std::io::Write;

fn main() -> Result<()> {
    {
        let _guard = TerminalGuard::new()?;
        writeln!(io::stdout(), "Hello, mein Freund!").context("Failed to write to stdout")?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    println!("Hello, Dashboard NMEA!");
    Ok(())
}
