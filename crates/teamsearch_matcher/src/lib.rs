//! Implementations and various utilities for the teamsearch string and pattern
//! matching.
//!
//! This crate is intended to serve as an abstraction layer between "actual"
//! searching and `teamsearch` itself. This means that the `teamsearch` tool can
//! remain to be somewhat implementation independent when its looking for
//! patterns within code.

use std::{fs::File, io::Read, path::PathBuf};

use anyhow::Result;
use derive_more::Constructor;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use serde::{self, ser::SerializeStruct, Serialize};

/// A match that was found within a file. This describes the
/// `range` of the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Constructor, Serialize)]
pub struct Match {
    /// The start of the match.
    pub start: usize,

    /// The end of the match.
    pub end: usize,
}

/// Internal structure used to report what `teamsearch` found with reference
/// to the `snippet` that was matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Constructor, Serialize)]
struct MatchSnippet<'s> {
    /// The start of the match.
    start: usize,

    /// The end of the match.
    end: usize,

    /// The text that was matched when scanning.
    #[serde(rename = "match")]
    snippet: &'s str,
}

/// The result of searching a file for matches.
#[derive(Debug, Clone, Default)]
pub struct FileMatches {
    /// The path to the file that was searched.
    pub path: PathBuf,

    /// The contents of the file that was scanned, this is useful for error
    /// reporting later on.
    pub contents: String,

    /// The matches that were found within the file.
    pub matches: Vec<Match>,
}

impl Serialize for FileMatches {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut item = serializer.serialize_struct("FileMatches", 2)?;
        item.serialize_field("path", &self.path)?;
        item.serialize_field("matches", &self.snippets())?;
        item.end()
    }
}

impl FileMatches {
    /// The number of matches that were found within the file.
    pub fn len(&self) -> usize {
        self.matches.len()
    }

    /// Whether or not the file has any matches.
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    fn snippets(&self) -> Vec<MatchSnippet> {
        self.matches
            .iter()
            .map(|m| MatchSnippet {
                start: m.start,
                end: m.end,
                snippet: &self.contents[m.start..m.end],
            })
            .collect()
    }
}

/// Perform a scan for a `pattern` of a given file, specified with a [PathBuf].
pub fn search_file(pattern: &str, path: PathBuf) -> Result<FileMatches> {
    let matcher = RegexMatcher::new(pattern)?;

    // Load the file contents.
    let contents = {
        let mut contents = String::new();
        File::open(&path)?.read_to_string(&mut contents)?;
        contents
    };

    let matches = find_matches(&matcher, &contents)?;
    Ok(FileMatches { path, contents, matches })
}

/// Find matches in a file.
///
///
/// This function will find all matches in a given file, and then return them
/// as a list of [Match]s.
fn find_matches(matcher: &RegexMatcher, contents: &str) -> Result<Vec<Match>> {
    let mut matches = Vec::new();

    let _ = matcher.try_find_iter::<_, std::io::Error>(contents.as_bytes(), |m| {
        matches.push(Match { start: m.start(), end: m.end() });
        Ok(true)
    })?;

    Ok(matches)
}
