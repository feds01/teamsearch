//! Implementation of the `find` command.

use std::{fs::File, path::PathBuf};

use anyhow::Result;
use teamsearch_workspace::{
    codeowners::CodeOwners,
    settings::Settings,
};

pub fn find(files: &[PathBuf], mut settings: Settings) -> Result<()> {
    let paths: Vec<PathBuf> = files.iter().map(fs::normalize_path).unique().collect();

    if paths.is_empty() {
        return Ok(());
    }

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &paths[0])?;

    if !codeowners.has_team(&settings.team) {
        // @@Todo: warn the user that there is no team.
        return Ok(());
    }
    Ok(())
}
}
