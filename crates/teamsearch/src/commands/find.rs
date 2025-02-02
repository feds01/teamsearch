//! Implementation of the `find` command.

use std::path::PathBuf;

use anyhow::Result;
use derive_more::Constructor;
use itertools::Itertools;
use log::debug;
use rayon::prelude::*;
use teamsearch_matcher::{FileMatches, search_file};
use teamsearch_utils::{fs, timed};
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::find_files_in_paths,
    settings::{FilePattern, Settings},
};

/// The result of a search.
#[derive(Default, Constructor)]
pub(crate) struct FindResult {
    /// The items that we're found within the files.
    pub file_matches: Vec<FileMatches>,
}

pub(crate) fn find(
    files: &[PathBuf],
    mut settings: Settings,
    team: Vec<String>,
    exclusions: Vec<String>,
    pattern: String,
) -> Result<FindResult> {
    let paths: Vec<PathBuf> = files.iter().map(fs::normalize_path).unique().collect();

    if paths.is_empty() {
        return Ok(FindResult::default());
    }

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &paths[0])?;

    let teams = team.iter().filter(|t| codeowners.has_team(t)).unique().collect::<Vec<_>>();

    // Augment the settings with the patterns.
    for team in &teams {
        let patterns = codeowners.get_patterns_for_team(team).to_vec();
        settings.file_resolver.include = settings.file_resolver.include.extend(patterns)?;
    }

    // Collect all of the paths that should be excluded.
    let mut all_exclusions = codeowners.get_ignored_patterns().to_vec();
    all_exclusions.extend(exclusions.iter().map(FilePattern::new_user));

    settings.file_resolver.user_exclude =
        settings.file_resolver.user_exclude.extend(all_exclusions)?;

    // Firstly, we need to discover all of the files in the provided paths.
    let files = timed(
        || find_files_in_paths(files, &settings),
        log::Level::Debug,
        |duration, result| {
            debug!("resolved {} files in {:?}", result.as_ref().map_or(0, |f| f.len()), duration)
        },
    )?;

    let mut matches: Vec<_> = files
        .into_par_iter()
        .map(|entry| -> Result<_, _> { search_file(&pattern, entry?.into_path()) })
        .filter(|result| {
            if let Ok(matches) = result {
                return !matches.is_empty();
            }

            true
        })
        .collect::<Result<Vec<_>, _>>()?;

    // We want to order the results by the "path" of the file, the match
    // contents will already be ordered by the line number.
    matches.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(FindResult::new(matches))
}
