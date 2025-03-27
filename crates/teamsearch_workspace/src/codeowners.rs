//! Implementation and utilities for dealing with the `CODEOWNERS
//! file format.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use derive_more::Constructor;

use crate::settings::{FilePattern, FilePatternSet};

#[derive(Debug, Constructor, Default)]
pub struct CodeOwners {
    /// The map of owners to the paths they own.
    pub owners: HashMap<String, Vec<FilePattern>>,

    /// A pre-computed matcher for the owner.
    owner_set: FilePatternSet,

    /// The root directory of the repository.
    root: PathBuf,

    /// Generally ignored paths.
    pub ignored_patterns: Vec<FilePattern>,
}

impl CodeOwners {
    /// Check whether the team exists or not.
    pub fn has_team(&self, team: &str) -> bool {
        self.owners.contains_key(team)
    }

    /// Get a [FilePatternSet] for a given team.
    fn get_pattern_for_team(&self, team: &str) -> FilePatternSet {
        FilePatternSet::try_from_iter(self.get_patterns_for_team(team).to_vec()).unwrap()
    }

    /// Check if a file is owned by a team.
    pub fn is_owned_by(&self, path: &Path, team: &str) -> bool {
        let set = self.get_pattern_for_team(team);

        // Get path relative to root if possible
        let relative_path = self.get_relative_path(path);
        let path_pat = self.format_path_for_matching(&relative_path);
        set.is_match(&path_pat)
    }

    /// Check if a file is owned by anyone.
    pub fn is_owned(&self, path: &Path) -> bool {
        let relative_path = self.get_relative_path(path);
        let path_pat = self.format_path_for_matching(&relative_path);

        self.owner_set.is_match(&path_pat)
    }

    /// Lookup a file path to see which team owns it.
    ///
    /// @@Future: we need to expand this so it supports multiple teams.
    /// The order should be based on the order of the teams in the CODEOWNERS
    /// file.
    pub fn lookup(&self, path: &Path) -> Vec<String> {
        let path = self.get_relative_path(path);
        let path_pat = self.format_path_for_matching(&path);

        let mut owners = vec![];

        for owner in self.owners.keys() {
            // @@Todo: we could potentially use a `OnceCell` here to cache the
            // pattern set for each team.
            let set = self.get_pattern_for_team(owner);

            if set.is_match(&path_pat) {
                owners.push(owner.clone());
            }
        }

        owners
    }

    /// Helper method to get a path relative to the root
    fn get_relative_path(&self, path: &Path) -> PathBuf {
        if path.starts_with(&self.root) {
            path.strip_prefix(&self.root).unwrap_or(path).to_path_buf()
        } else {
            path.to_path_buf()
        }
    }

    /// Helper method to format path for matching, ensuring directories end with
    /// "/"
    /// Helper method to format path for matching
    /// - Ensures directories end with "/"
    /// - Ensures paths start with "/"
    fn format_path_for_matching(&self, path: &Path) -> String {
        // Convert path to string
        let mut path_str = path.to_string_lossy().to_string();

        // Ensure directories end with "/"
        if path.is_dir() && !path_str.ends_with('/') {
            path_str = format!("{}/", path_str);
        }

        // Ensure paths start with "/"
        if !path_str.starts_with('/') {
            path_str = format!("/{}", path_str);
        }

        path_str
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
    pub fn parse_from_file(path: &PathBuf, root: &Path) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path).or_else(|_| {
            anyhow::bail!("Failed to read the CODEOWNERS file at {:?}", path);
        })?;

        let mut owners = CodeOwners { root: root.to_path_buf(), ..CodeOwners::default() };

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

                buf.to_string()
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

        // Now compute the matcher for the owners.
        owners.owner_set =
            FilePatternSet::try_from_iter(owners.owners.values().flatten().cloned()).unwrap();

        Ok(owners)
    }
}
