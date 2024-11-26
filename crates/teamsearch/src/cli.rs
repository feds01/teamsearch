//! Definitions of the command line interface for the `bl` binary.


use clap::{command, Parser};

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
    /// Command to print the version of the `bl` binary.
    Version,
}
