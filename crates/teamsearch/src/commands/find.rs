//! Implementation of the `find` command.

use std::{iter::once, path::PathBuf};

use anyhow::Result;
use derive_more::Constructor;
use itertools::Itertools;
use log::debug;
use rayon::prelude::*;
use teamsearch_matcher::{FileMatches, Pattern, search_file};
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
    case_sensitive: bool,
) -> Result<FindResult> {
    let paths: Vec<PathBuf> =
        files.iter().chain(once(&settings.codeowners)).map(fs::normalize_path).unique().collect();

    if paths.is_empty() {
        return Ok(FindResult::default());
    }

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let root = fs::common_root(&paths);
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;
    let teams = team.iter().filter(|t| codeowners.has_team(t)).unique().collect::<Vec<_>>();

    // If we get no teams at all, we assume that we're doing a wide scan
    // across an entire repo. This is useful for other modes of scanning that
    // are looking for things across all teams, or for a specific pattern.
    if teams.is_empty() {
        settings.file_resolver.include =
            settings.file_resolver.include.extend(vec![FilePattern::all()])?;
    }

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
        .map(|entry| -> Result<_, _> {
            search_file(Pattern::new(&pattern, case_sensitive), entry?.into_path())
        })
        .filter(|result| {
            if let Ok(matches) = result {
                return !matches.is_empty();
            }

            false
        })
        .collect::<Result<Vec<_>, _>>()?;

    // We want to order the results by the "path" of the file, the match
    // contents will already be ordered by the line number.
    matches.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(FindResult::new(matches))
}
