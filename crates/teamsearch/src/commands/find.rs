//! Implementation of the `find` command.

use std::{iter::once, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use log::{debug, warn};
use rayon::prelude::*;
use teamsearch_matcher::{FileMatches, Pattern, search_file};
use teamsearch_utils::{fs, timed};
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::find_files_in_paths,
    settings::{FilePattern, Settings},
};

/// The result of a search.
#[derive(Default)]
pub(crate) struct FindResult {
    /// The items that we're found within the files.
    pub file_matches: Vec<FileMatches>,
}

impl FindResult {
    pub(crate) fn new(file_matches: Vec<FileMatches>) -> Self {
        Self { file_matches }
    }
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

    let root = fs::common_root(&paths);

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &root)?;
    let (teams, unknown_teams): (Vec<_>, Vec<_>) =
        team.iter().unique().partition(|t| codeowners.has_team(t));

    for team in &unknown_teams {
        warn!("`{team}` is not a team in {}, ignoring it", settings.codeowners.display());
    }

    // If we get no teams at all, we assume that we're doing a wide scan
    // across an entire repo. This is useful for other modes of scanning that
    // are looking for things across all teams, or for a specific pattern.
    if teams.is_empty() {
        if !unknown_teams.is_empty() {
            warn!(
                "none of the given teams are in {}, searching every file",
                settings.codeowners.display()
            );
        }

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
        || find_files_in_paths(&root, files, &settings),
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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    /// Search `root` for "needle" in the files owned by `teams`, returning
    /// the matching paths relative to `root`.
    fn find_in(root: &Path, teams: &[&str]) -> Vec<String> {
        let settings = Settings::new(true, root.join("CODEOWNERS"));
        let teams = teams.iter().map(|team| team.to_string()).collect();

        find(&[root.to_path_buf()], settings, teams, vec![], "needle".into(), false)
            .unwrap()
            .file_matches
            .iter()
            .map(|file| file.path.strip_prefix(root).unwrap().display().to_string())
            .collect()
    }

    #[test]
    fn test_find_skips_unknown_teams() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let root = temp_dir.path().canonicalize().unwrap();

        for (path, contents) in [
            ("CODEOWNERS", "/web/ @acme/frontend\n/api/ @acme/backend\n"),
            ("web/app.js", "needle\n"),
            ("api/main.py", "needle\n"),
        ] {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }

        assert_eq!(find_in(&root, &["@acme/frontend", "@acme/nope"]), vec!["web/app.js"]);
        assert_eq!(
            find_in(&root, &["@acme/nope", "@acme/nope"]),
            vec!["api/main.py", "web/app.js"]
        );
    }
}
