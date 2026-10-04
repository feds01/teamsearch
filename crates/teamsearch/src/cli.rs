//! Definitions of the command line interface for the `teamsearch` binary.

use std::{
    ffi::{OsStr, OsString},
    io, iter,
    path::PathBuf,
};

use argfile::{Argument, PREFIX};
use clap::Parser;

/// Expand `@path` argfiles in `args`. An `@` argument that doesn't name an
/// existing file, such as a CODEOWNERS team like `@org/team`, is kept as-is.
pub fn expand_args(args: impl IntoIterator<Item = OsString>) -> io::Result<Vec<OsString>> {
    let mut expanded = Vec::new();

    for arg in args {
        match classify_arg(&arg) {
            Argument::Path(_) => {
                expanded.extend(argfile::expand_args_from(iter::once(arg), parse_argfile, PREFIX)?);
            }
            Argument::PassThrough(arg) => expanded.push(arg),
        }
    }

    Ok(expanded)
}

/// Parse an argfile holding one argument per line.
fn parse_argfile(content: &str, _prefix: char) -> Vec<Argument> {
    content.lines().map(classify_arg).collect()
}

/// Treat `arg` as an argfile only when it is `@path` and `path` is a file.
fn classify_arg(arg: impl AsRef<OsStr>) -> Argument {
    match Argument::parse_ref(&arg, PREFIX) {
        Argument::Path(path) if path.is_file() => Argument::Path(path),
        _ => Argument::PassThrough(arg.as_ref().to_owned()),
    }
}

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

    /// Find orphaned files that don't belong to any team.
    Orphans(OrphanCommand),

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

    /// Treat the pattern as case insensitive.
    #[clap(
        short = 'i',
        long,
        help = "Treat the pattern as case insensitive",
        default_value = "false"
    )]
    pub case_insensitive: bool,

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
    #[clap(long, help = "Display the results using in JSON format")]
    pub json: bool,

    /// Whether to simply output the counts of the matches per file.
    #[clap(long, help = "Output the counts of the matches per file")]
    pub count: bool,
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

    /// Display the results using a JSON format. We output the contents
    /// of the search in the following format:
    ///
    /// ```json
    /// [
    ///     {
    ///         "path": "some/foo/result.rs",
    ///         "team": "@some-team"
    ///     },
    ///     {
    ///         "path": "some/bar/result.rs",
    ///         "team": null
    ///     },
    /// ]
    /// ```
    #[clap(long, help = "Display the results using in JSON format")]
    pub json: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct OrphanCommand {
    /// List of directories that should be checked for orphans.
    #[clap(help = "List of files or directories to check [default: .]")]
    pub files: Vec<PathBuf>,

    /// Specify the path of the file of the codeowners.
    #[clap(long, short, help = "Specify the path of the CODEOWNERS file [default: CODEOWNERS]")]
    pub codeowners: PathBuf,

    /// Paths that should be excluded from the search.
    #[clap(
        long,
        short,
        help = "Paths that should be excluded from the search [default: none]",
        value_name = "PATH"
    )]
    pub exclude: Vec<String>,

    /// Display the results using a JSON format. We output the contents
    /// of the search in the following format:
    ///
    /// ```json
    /// [
    ///     "some/foo/orphan.rs",
    ///     "some/bar/orphan.rs",
    /// ]
    /// ```
    #[clap(long, help = "Display the results using in JSON format")]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    fn parse_teams(args: Vec<OsString>) -> Vec<String> {
        match Cli::try_parse_from(args).expect("failed to parse args").command {
            Command::Find(find) => find.teams,
            other => panic!("expected `find`, got {other:?}"),
        }
    }

    #[test]
    fn test_team_args_are_not_treated_as_argfiles() {
        let args = ["teamsearch", "find", "-c", "CODEOWNERS", "-p", "needle", "-t", "@org/backend"]
            .map(OsString::from);

        assert_eq!(parse_teams(expand_args(args).unwrap()), vec!["@org/backend"]);
    }

    #[test]
    fn test_argfiles_are_expanded() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let nested_path = temp_dir.path().join("nested.txt");
        fs::write(&nested_path, "-t\n@org/frontend\n").expect("Failed to write nested argfile");

        let argfile_path = temp_dir.path().join("args.txt");
        fs::write(&argfile_path, format!("-p\nneedle\n@{}\n", nested_path.display()))
            .expect("Failed to write argfile");

        let mut argfile = OsString::from("@");
        argfile.push(&argfile_path);

        let args = [
            "teamsearch".into(),
            "find".into(),
            "-c".into(),
            "CODEOWNERS".into(),
            argfile,
            "-t".into(),
            "@org/backend".into(),
        ];

        assert_eq!(parse_teams(expand_args(args).unwrap()), vec!["@org/frontend", "@org/backend"]);
    }
}
