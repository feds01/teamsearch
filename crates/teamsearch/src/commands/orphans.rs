use std::{iter::once, path::PathBuf};

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use teamsearch_utils::{fs, thread_pool};
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

    // Add exclusions from the user:
    settings.file_resolver.user_exclude =
        settings.file_resolver.user_exclude.extend(exclusions.iter().map(FilePattern::new_user))?;

    settings.file_resolver.include =
        settings.file_resolver.include.extend(vec![FilePattern::all()])?;

    let all_files =
        find_files_in_paths(files, &settings)?.into_iter().collect::<Result<Vec<_>, _>>()?;

    // Depending on whether we have a small number of files, we can either
    // use a thread pool or not. Typically, for small numbers of files, we
    // don't need to use a thread pool.
    let orphans = match all_files.len() {
        0..=1000 => {
            all_files.into_iter().filter(|file| !codeowners.is_owned(file.path())).collect_vec()
        }
        _ => {
            // Construct a thread pool with limited threads.
            //
            // For a small number of files, there no need to use a thread pool.
            let pool = thread_pool::construct_thread_pool();
            pool.install(|| {
                all_files
                    .par_iter()
                    .filter(|file| !codeowners.is_owned(file.path()))
                    .cloned()
                    .collect()
            })
        }
    };

    Ok(OrphanResult { orphans })
}
