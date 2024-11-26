//! Implementation of the `find` command.

use std::{fs::File, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use log::info;
use teamsearch_utils::{fs, timed};
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::{find_files_in_paths, ResolvedFile},
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

    // Augment the settings with the patterns.
    let patterns = codeowners.get_patterns_for_team(&settings.team).to_vec();
    settings.file_resolver.include = settings.file_resolver.include.extend(patterns)?;
    settings.file_resolver.user_exclude =
        settings.file_resolver.user_exclude.extend(codeowners.get_ignored_patterns().to_vec())?;

    // Firstly, we need to discover all of the files in the provided paths.
    let files = timed(
        || find_files_in_paths(files, &settings),
        log::Level::Info,
        |duration, result| {
            info!("Resolved {} files in {:?}", result.as_ref().map_or(0, |f| f.len()), duration)
        },
    )?;

    if files.is_empty() {
        // @@Todo: warn the user that there were no files to check.
        return Ok(());
    }
    Ok(())
}
}
