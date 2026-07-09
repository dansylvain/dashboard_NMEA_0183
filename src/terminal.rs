use anyhow::{Context, Result};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io;

pub struct TerminalGuard {}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        execute!(io::stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;
        Ok(Self {})
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen)
            .context("Failed to exit alternate screen: ")
        {
            eprintln!("{e}");
        }
        if let Err(e) = disable_raw_mode().context("failed to disable raw mode: ") {
            eprintln!("{e}");
        }
    }
}
