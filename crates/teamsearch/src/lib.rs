//! Library definition of `teamsearch` crate.

#![feature(panic_payload_as_str)]

pub mod cli;
mod commands;
pub(crate) mod version;

use std::{
    process::ExitCode,
};

use anyhow::{anyhow, Ok, Result};
use cli::FindCommand;
#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// Linting was successful and there were no linting errors.
    Success,
    /// Linting was successful but there were linting errors.
    Failure,
    /// Linting failed.
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
        }
    }
}
/// Handler function which will delegate functionality to the appropriate
/// command.
pub fn run(cli::Cli { command }: cli::Cli) -> Result<ExitStatus> {
    match command {
        cli::Command::Version => version(),
    }
}

fn version() -> Result<ExitStatus> {
    commands::version::version()?;
    Ok(ExitStatus::Success)
}
