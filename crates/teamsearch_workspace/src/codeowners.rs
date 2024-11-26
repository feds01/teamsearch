//! Implementation and utilities for dealing with the `CODEOWNERS
//! file format.

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use derive_more::Constructor;
use teamsearch_utils::fs;

use crate::settings::FilePattern;

#[derive(Debug, Constructor, Default)]
pub struct CodeOwners {
    /// The map of owners to the paths they own.
    pub owners: HashMap<String, Vec<FilePattern>>,

    /// Generally ignored paths.
    pub ignored_patterns: Vec<FilePattern>,
}

impl CodeOwners {
    /// Check whether the team exists or not.
    pub fn has_team(&self, team: &str) -> bool {
        self.owners.contains_key(team)
    }

    /// Get all patterns for a specific team.
    pub fn get_patterns_for_team(&self, team: &str) -> &[FilePattern] {
        self.owners.get(team).map_or(&[], |v| v)
    }

    /// Get the ignored patterns.
    pub fn get_ignored_patterns(&self) -> &[FilePattern] {
        &self.ignored_patterns
    }

    /// Parse the contents of the CODEOWNERS file. This file format is very
    /// simple, the basics are as follows:
    ///
    /// - Comments are lines that start with `#`.
    ///
    /// - Each line is a path, followed by a list of owners.
    ///
    /// - If no owners are specified, we consider these to be owned by anyone.
    ///   For the purpose of this tool, we can openly "ignore" these paths,
    ///   since we want to look at files that a particular team owns.
    ///
    /// Example: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners#example-of-a-codeowners-file
    ///
    /// ```plaintext
    /// 
    /// # This is a comment.
    ///
    /// # The owners of the root directory.
    ///
    /// /some_directory/ @some-team
    ///
    /// # The owners of the `docs` directory.
    ///
    /// /docs/ @another-team @some-team
    /// ```
    pub fn parse_from_file(path: &PathBuf, root: &PathBuf) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path).or_else(|_| {
            anyhow::bail!("Failed to read the CODEOWNERS file at {:?}", path);
        })?;

        let mut owners = CodeOwners::default();

        for line in contents.lines() {
            let line = line.trim();

            // Skip empty lines and comments.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let path = parts.next().unwrap();
            let owners_annotations: Vec<String> = parts.map(str::to_string).collect();

            let convert_to_user = |path: &str| {
                let mut buf = path.to_owned();

                if buf.ends_with('/') {
                    buf.push_str("**");
                }

                fs::normalize_path_to(buf, root).to_string_lossy().to_string()
            };

            // If no owners are specified, we consider these to be owned by anyone, and
            // hence we can actually ignore this path.
            if owners_annotations.is_empty() {
                owners.ignored_patterns.push(FilePattern::User(convert_to_user(path)));
                continue;
            }

            // Update all of the owners for the given path.
            for owner in owners_annotations {
                let abs = convert_to_user(path);
                owners.owners.entry(owner).or_default().push(FilePattern::User(abs));
            }
        }

        Ok(owners)
    }
}
