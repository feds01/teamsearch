//! Definitions of the command line interface for the `teamsearch` binary.

use std::path::PathBuf;

use clap::{Parser, command};

#[derive(Debug, Parser)]
#[command(
    author,
    name = "teamsearch",
    about = "TeamSearch: Search for large code bases with ease using CODEOWNERS",
    after_help = "For help with a specific command, see: `teamsearch help <command>`."
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Find the code that you're looking for based on the CODEOWNERS file.
    Find(FindCommand),

    /// Lookup the team that owns a specific file or directory.
    Lookup(LookupCommand),

    /// Command to print the version of the `teamsearch` binary.
    Version,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct FindCommand {
    /// List of files or directories to check.
    #[clap(help = "List of files or directories to check [default: .]")]
    pub files: Vec<PathBuf>,

    /// Respect file exclusions via `.gitignore` and other standard ignore
    /// files. Use `--no-respect-gitignore` to disable.
    #[arg(
        long,
        overrides_with("no_respect_gitignore"),
        help_heading = "File selection",
        default_value = "true"
    )]
    pub respect_gitignore: bool,

    #[clap(long, overrides_with("respect_gitignore"), hide = true)]
    no_respect_gitignore: bool,

    /// Specify the path of the file of the codeowners.
    #[clap(long, short, help = "Specify the path of the CODEOWNERS file [default: CODEOWNERS]")]
    pub codeowners: PathBuf,

    /// Specify the team to check for.
    #[clap(value_parser = parse_team_name, long, short, help = "Specify the team to check for [default: *]")]
    pub teams: Vec<String>,

    /// Paths that should be excluded from the search.
    #[clap(
        long,
        short,
        help = "Paths that should be excluded from the search [default: none]",
        value_name = "PATH"
    )]
    pub exclude: Vec<String>,

    /// The pattern to look for within the codebase.
    #[clap(short)]
    pub pattern: String,

    /// Display the results using a JSON format. We output the contents
    /// of the search in the following format:
    ///
    /// ```json
    /// [
    ///     {
    ///         "path": "some/foo/result.rs",
    ///         "matches": [
    ///             "start": 0,
    ///             "end": 11,
    ///             "match": "hello world"
    ///         ]
    ///     }
    /// ]
    /// ```
    #[clap(long, help = "Display the results using a JSON format")]
    pub json: bool,
}

fn parse_team_name(raw_team: &str) -> Result<String, String> {
    if raw_team.starts_with('@') { Ok(raw_team.to_string()) } else { Ok(format!("@{}", raw_team)) }
}

#[derive(Clone, Debug, clap::Parser)]
pub struct LookupCommand {
    /// List of files to check to which team they belong to.
    #[clap(help = "List of files or directories to check [default: .]")]
    pub files: Vec<PathBuf>,

    /// Specify the path of the file of the codeowners.
    #[clap(long, short, help = "Specify the path of the CODEOWNERS file [default: CODEOWNERS]")]
    pub codeowners: PathBuf,
}
