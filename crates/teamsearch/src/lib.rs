//! Library definition of `teamsearch` crate.

#![feature(panic_payload_as_str)]

pub mod cli;
mod commands;
mod crash;
pub(crate) mod version;

use std::{
    collections::BTreeMap,
    panic,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Ok, Result, anyhow};
use cli::{FindCommand, LookupCommand, OrphanCommand};
use commands::{find::FindResult, lookup::LookupEntry};
use crash::crash_handler;
use log::info;
use teamsearch_matcher::Match;
use teamsearch_utils::{lines::get_line_range, logging::ToolLogger, stream::CompilerOutputStream};
use teamsearch_workspace::settings::Settings;

#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// Scanning was successful and there were no errors.
    Success,

    /// Scanning was successful but there were errors.
    Failure,

    /// Scanning failed.
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
        cli::Command::Orphans(args) => orphans(args),
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

    let start = std::time::Instant::now();

    let settings = Settings::new(args.respect_gitignore, args.codeowners);
    let FindResult { file_matches } = commands::find::find(
        &files,
        settings,
        args.teams,
        args.exclude,
        args.pattern,
        args.case_insensitive,
    )?;

    // Now, we need to print out the results based on the configuration of the user.
    if args.json {
        // Print out the results in JSON format.
        println!("{}", serde_json::to_string_pretty(&file_matches)?);
    } else {
        for (idx, result) in file_matches.iter().enumerate() {
            if args.count {
                info!("{}: {}", result.path.display(), result.len());
            } else {
                // Group matches by line number to avoid printing duplicate lines.
                // BTreeMap automatically keeps lines sorted by line number.
                let line_matches: BTreeMap<usize, String> = result
                    .matches
                    .iter()
                    .map(|m| {
                        let (line_num, line_content) = get_line_info(&result.contents, m.start);
                        (line_num, line_content)
                    })
                    .collect();
                
                // Print file path followed by all matching lines.
                if !line_matches.is_empty() {
                    println!("{}", result.path.display());
                    
                    for (line_num, line_content) in &line_matches {
                        println!("{}:{}", line_num, line_content);
                    }

                    // Only print blank line between files, not after the last one.
                    if idx < file_matches.len() - 1 {
                        println!();
                    }
                }
            }
        }

        let total_matches = file_matches.iter().map(|m| m.len()).sum::<usize>();
        info!("found {} matches in {:?}", total_matches, start.elapsed());
    }

    Ok(ExitStatus::Success)
}

fn lookup(args: LookupCommand) -> Result<ExitStatus> {
    let files = resolve_default_files(args.files, false);

    // Ensure that the codeowners file is present.
    if !args.codeowners.exists() {
        return Err(anyhow!("The CODEOWNERS file does not exist."));
    }

    let settings = Settings::new(true, args.codeowners);
    let results = commands::lookup::lookup(&files, settings)?;

    if args.json {
        // Print out the results in JSON format.
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for LookupEntry { path, teams } in results.entries {
            //
            if teams.is_empty() {
                info!("{}: none", path.display());
                continue;
            }

            for team in teams {
                info!("{}: {}", path.display(), team)
            }
        }
    }

    Ok(ExitStatus::Success)
}

fn orphans(args: OrphanCommand) -> Result<ExitStatus> {
    let files = resolve_default_files(args.files, false);

    // Ensure that the codeowners file is present.
    if !args.codeowners.exists() {
        return Err(anyhow!("The CODEOWNERS file does not exist."));
    }

    let start = std::time::Instant::now();
    let settings = Settings::new(true, args.codeowners);
    let results = commands::orphans::orphans(&files, settings, args.exclude)?;

    if args.json {
        // Print out the results in JSON format.
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for result in &results.orphans {
            info!("{}", result.path().display())
        }

        info!("found {} files in {:?}", results.orphans.len(), start.elapsed());
    }

    Ok(ExitStatus::Success)
}

fn version() -> Result<ExitStatus> {
    commands::version::version()?;
    Ok(ExitStatus::Success)
}
