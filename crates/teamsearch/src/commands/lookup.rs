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

    let files: Vec<PathBuf> = files.iter().map(fs::normalize_path).unique().collect();

    // Compute the "root" of all of the paths including the provided paths and the
    // CODEOWNERS file.
    let root_paths: Vec<PathBuf> =
        files.iter().cloned().chain(once(fs::normalize_path(&settings.codeowners))).collect();
    let root = fs::common_root(&root_paths);

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;

    let entries = files
        .into_iter()
        .map(|path| LookupEntry { teams: codeowners.lookup(&path), path })
        .collect();

    Ok(LookupResult { entries })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_lookup_keeps_every_file_when_codeowners_is_one_of_them() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let codeowners_path = temp_dir.path().join("CODEOWNERS");
        fs::write(&codeowners_path, "* @devs\n/src/ @dev-team\n")
            .expect("Failed to write CODEOWNERS file");

        let files = vec![
            temp_dir.path().join("src/main.rs"),
            codeowners_path.clone(),
            temp_dir.path().join("docs/README.md"),
        ];

        let result = lookup(&files, Settings::new(true, codeowners_path)).unwrap();
        let paths: Vec<&PathBuf> = result.entries.iter().map(|entry| &entry.path).collect();

        let teams: Vec<Vec<&str>> = result
            .entries
            .iter()
            .map(|entry| entry.teams.iter().map(String::as_str).sorted().collect())
            .collect();

        assert_eq!(paths, files.iter().collect::<Vec<_>>());
        assert_eq!(teams, vec![vec!["@dev-team", "@devs"], vec!["@devs"], vec!["@devs"]]);
    }
}
