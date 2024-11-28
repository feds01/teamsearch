//! Library definition of `teamsearch` crate.

#![feature(panic_payload_as_str)]

pub mod cli;
mod commands;
mod crash;
pub(crate) mod version;

use std::{
    panic,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{anyhow, Ok, Result};
use cli::{FindCommand, LookupCommand};
use crash::crash_handler;
use teamsearch_utils::{logging::ToolLogger, stream::CompilerOutputStream};
use teamsearch_workspace::settings::Settings;

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

pub static LOGGER: ToolLogger = ToolLogger::new();

/// Handler function which will delegate functionality to the appropriate
/// command.
pub fn run(cli::Cli { command }: cli::Cli) -> Result<ExitStatus> {
    // Initial grunt work, panic handler and logger setup...
    panic::set_hook(Box::new(crash_handler));

    let output_stream = CompilerOutputStream::stdout;
    let error_stream = CompilerOutputStream::stderr;

    log::set_logger(&LOGGER).unwrap_or_else(|_| panic!("couldn't initiate logger"));

    LOGGER.error_stream.set(error_stream()).unwrap();
    LOGGER.output_stream.set(output_stream()).unwrap();

    // If we're building in debug mode, we want to see all the logs.
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Debug);

    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);

    // We also need to create a global-config

    match command {
        cli::Command::Find(args) => find(args),
        cli::Command::Lookup(args) => lookup(args),
        cli::Command::Version => version(),
    }
}

/// Returns the default set of files if none are provided, otherwise returns
/// `None`.
fn resolve_default_files(files: Vec<PathBuf>, is_stdin: bool) -> Vec<PathBuf> {
    if files.is_empty() {
        if is_stdin {
            vec![Path::new("-").to_path_buf()]
        } else {
            vec![Path::new(".").to_path_buf()]
        }
    } else {
        files
    }
}

fn find(args: FindCommand) -> Result<ExitStatus> {
    let files = resolve_default_files(args.files, false);

    // Ensure that the codeowners file is present.
    if !args.codeowners.exists() {
        return Err(anyhow!("The CODEOWNERS file does not exist."));
    }

    let settings = Settings::new(args.respect_gitignore, args.codeowners);
    commands::find::find(&files, settings, args.teams, args.pattern)?;

    Ok(ExitStatus::Success)
}

fn lookup(args: LookupCommand) -> Result<ExitStatus> {
    let files = resolve_default_files(args.files, false);

    // Ensure that the codeowners file is present.
    if !args.codeowners.exists() {
        return Err(anyhow!("The CODEOWNERS file does not exist."));
    }

    let settings = Settings::new(true, args.codeowners);
    commands::lookup::lookup(&files, settings)?;

    Ok(ExitStatus::Success)
}

fn version() -> Result<ExitStatus> {
    commands::version::version()?;
    Ok(ExitStatus::Success)
}
