//! Implementation of the `find` command.

use std::{fs::File, path::PathBuf};

use anyhow::Result;
use grep_printer::{ColorSpecs, Standard, StandardBuilder};
use grep_regex::RegexMatcher;
use grep_searcher::SearcherBuilder;
use itertools::Itertools;
use log::info;
use teamsearch_utils::{fs, timed};
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::{find_files_in_paths, ResolvedFile},
    settings::{FilePattern, Settings},
};
use termcolor::StandardStream;

pub fn find(
    files: &[PathBuf],
    mut settings: Settings,
    team: Vec<String>,
    exclusions: Vec<String>,
    pattern: String,
) -> Result<()> {
    let paths: Vec<PathBuf> = files.iter().map(fs::normalize_path).unique().collect();

    if paths.is_empty() {
        return Ok(());
    }

    // We've gotta parse in the `CODEOWNERS` file, and then
    // extract the given patterns that are specified for the particular team.
    let codeowners = CodeOwners::parse_from_file(&settings.codeowners, &paths[0])?;

    let teams = team.iter().filter(|t| codeowners.has_team(t)).unique().collect::<Vec<_>>();

    if teams.is_empty() {
        // @@Todo: warn the user that there is no team.
        return Ok(());
    }

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
        log::Level::Info,
        |duration, result| {
            info!("Resolved {} files in {:?}", result.as_ref().map_or(0, |f| f.len()), duration)
        },
    )?;

    if files.is_empty() {
        // @@Todo: warn the user that there were no files to check.
        return Ok(());
    }

    // @@Todo: integrate a cache system here, we should be able to avoid re-linting
    // already existent files and just skip them.
    let matcher = RegexMatcher::new(pattern.as_str())?;

    let mut printer = StandardBuilder::new()
        .heading(true)
        .stats(true)
        .path(true)
        .color_specs(ColorSpecs::default_with_color())
        .build(termcolor::StandardStream::stdout(termcolor::ColorChoice::Always));

    for entry in files {
        match entry? {
            ResolvedFile::Nested(file) | ResolvedFile::Root(file) => {
                let _ = find_matches(&matcher, &file, &mut printer);
            }
        }
    }

    Ok(())
}

/// Use the `grep` crate to find matches in a file.
///
/// We aggregate the matches into a vector of tuples, where the first element
/// is the line number and the second element is the line itself.
fn find_matches(
    matcher: &RegexMatcher,
    path: &PathBuf,
    printer: &mut Standard<StandardStream>,
) -> Result<()> {
    let mut searcher =
        SearcherBuilder::new().multi_line(true).before_context(1).after_context(1).build();

    let file = File::open(path)?;
    searcher.search_file(matcher, &file, printer.sink_with_path(&matcher, path))?;

    Ok(())
}
