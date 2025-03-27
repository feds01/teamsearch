use std::{iter::once, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use teamsearch_utils::fs;
use teamsearch_workspace::{codeowners::CodeOwners, settings::Settings};

/// An lookup entry, representing a file and its corresponding
/// owners.
#[derive(Serialize)]
pub(crate) struct LookupEntry {
    /// The owner of the file, if any.
    pub(crate) teams: Vec<String>,

    /// The path of the entry.
    pub(crate) path: PathBuf,
}

/// The result of an owner lookup.
#[derive(Serialize, Default)]
#[serde(transparent)]
pub(crate) struct LookupResult {
    pub(crate) entries: Vec<LookupEntry>,
}

pub fn lookup(files: &[PathBuf], settings: Settings) -> Result<LookupResult> {
    if files.is_empty() {
        return Ok(LookupResult::default());
    }

    // Compute the "root" of all of the paths including the provided paths and the
    // CODEOWNERS file.
    let paths: Vec<PathBuf> =
        files.iter().chain(once(&settings.codeowners)).map(fs::normalize_path).unique().collect();
    let root = fs::common_root(&paths);

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;

    let mut entries = Vec::new();

    // For each path (other than last), we need to find the team that owns it.
    for path in paths.iter().take(paths.len() - 1) {
        entries.push(LookupEntry { path: path.clone(), teams: codeowners.lookup(path) });
    }

    Ok(LookupResult { entries })
}
