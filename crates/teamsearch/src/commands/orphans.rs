use std::{iter::once, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use teamsearch_utils::fs;
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::{ResolvedFile, find_files_in_paths},
    settings::{FilePattern, Settings},
};

/// The result of looking for orphans.
#[derive(Serialize, Default, Debug)]
#[serde(transparent)]
pub(crate) struct OrphanResult {
    pub(crate) orphans: Vec<ResolvedFile>,
}

pub fn orphans(
    files: &[PathBuf],
    mut settings: Settings,
    exclusions: Vec<String>,
) -> Result<OrphanResult> {
    if files.is_empty() {
        return Ok(OrphanResult::default());
    }

    // Compute the "root" of all of the paths including the provided paths and the
    // CODEOWNERS file.
    let paths: Vec<PathBuf> =
        files.iter().chain(once(&settings.codeowners)).map(fs::normalize_path).unique().collect();
    let root = fs::common_root(&paths);

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;

    // Add as an "exclude" all of the patterns from the teams:
    for team in codeowners.owners.keys() {
        let patterns = codeowners.get_patterns_for_team(team).iter().cloned();
        settings.file_resolver.exclude = settings.file_resolver.exclude.extend(patterns)?;
    }

    // Add exclusions from the user:
    settings.file_resolver.user_exclude =
        settings.file_resolver.user_exclude.extend(exclusions.iter().map(FilePattern::new_user))?;

    settings.file_resolver.include =
        settings.file_resolver.include.extend(vec![FilePattern::all()])?;

    // Firstly, we need to discover all of the files in the provided paths.
    let orphans =
        find_files_in_paths(files, &settings)?.into_iter().collect::<Result<Vec<_>, _>>()?;

    Ok(OrphanResult { orphans })
}
