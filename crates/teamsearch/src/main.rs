//! Entry point of teamsearch.

use std::process::ExitCode;

use clap::Parser;
use teamsearch::{
    ExitStatus,
    cli::{Cli, expand_args},
    run,
};

pub fn main() -> ExitCode {
    // Enabled ANSI colours on Windows 10.
    #[cfg(windows)]
    assert!(colored::control::set_virtual_terminal(true).is_ok());

    let args = match expand_args(wild::args_os()) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitStatus::Error.into();
        }
    };

    let args = Cli::parse_from(args);

    match run(args) {
        Ok(status) => status.into(),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
