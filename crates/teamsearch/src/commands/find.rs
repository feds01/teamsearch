//! Implementation of the `find` command.
use core::str;
use std::{fs::File, io::Read, path::PathBuf, time::Instant};

use annotate_snippets::{Level, Renderer, Snippet};
use anyhow::Result;
use grep_matcher::{Match, Matcher};
use grep_regex::RegexMatcher;
use itertools::Itertools;
use log::info;
use teamsearch_utils::{fs, timed};
use teamsearch_workspace::{
    codeowners::CodeOwners,
    resolver::{find_files_in_paths, ResolvedFile},
    settings::{FilePattern, Settings},
};

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
            info!("resolved {} files in {:?}", result.as_ref().map_or(0, |f| f.len()), duration)
        },
    )?;

    if files.is_empty() {
        // @@Todo: warn the user that there were no files to check.
        return Ok(());
    }

    // @@Todo: integrate a cache system here, we should be able to avoid re-linting
    // already existent files and just skip them.
    let renderer = Renderer::styled();

    let matcher = RegexMatcher::new(pattern.as_str())?;
    let start = Instant::now();
    let mut total_matches = 0;

    for entry in files {
        match entry? {
            ResolvedFile::Nested(file) | ResolvedFile::Root(file) => {
                let contents = {
                    let mut contents = String::new();
                    File::open(&file)?.read_to_string(&mut contents)?;
                    contents
                };

                let matches = find_matches(&matcher, &contents)?;
                total_matches += matches.len();

                // Print each match using annotate-snippets
                for m in matches {
                    let level = Level::Info;
                    let message = level.title("match found").snippet(
                        Snippet::source(contents.as_str())
                            .origin(file.as_os_str().to_str().unwrap())
                            .fold(true)
                            .annotation(level.span(m.start()..m.end()).label("")),
                    );

                    println!("{}", renderer.render(message));
                }
            }
        }
    }

    info!("found {} matches in {:?}", total_matches, start.elapsed());
    Ok(())
}

/// Use the `grep` crate to find matches in a file.
///
/// We aggregate the matches into a vector of tuples, where the first element
/// is the line number and the second element is the line itself.
fn find_matches(matcher: &RegexMatcher, contents: &str) -> Result<Vec<Match>> {
    let mut matches = Vec::new();

    let _ = matcher.try_find_iter::<_, std::io::Error>(contents.as_bytes(), |m| {
        matches.push(m);
        Ok(true)
    })?;

    Ok(matches)
}
