use std::{iter::once, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use teamsearch_utils::fs;
use teamsearch_workspace::{codeowners::CodeOwners, settings::Settings};

pub fn lookup(files: &[PathBuf], settings: Settings) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    // Compute the "root" of all of the paths including the provided paths and the
    // CODEOWNERS file.
    let paths: Vec<PathBuf> =
        files.iter().chain(once(&settings.codeowners)).map(fs::normalize_path).unique().collect();
    let root = fs::common_root(&paths);

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;

    // For each path (other than last), we need to find the team that owns it.
    for path in paths.iter().take(paths.len() - 1) {
        let team = codeowners.lookup(path).unwrap_or("none");
        println!("{}: {}", path.display(), team);
    }

    Ok(())
}
