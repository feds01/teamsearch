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
    fn format_path_for_matching(&self, path: &Path) -> String {
        // Convert path to string
        let mut path_str = path.to_string_lossy().to_string();

        // Ensure directories end with "/"
        if path.is_dir() && !path_str.ends_with('/') {
            path_str = format!("{}/", path_str);
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

                // if the path starts with a `/`, we need to remove it.
                if buf.starts_with('/') {
                    buf = buf[1..].to_string();
                }

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

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    use tempfile::{TempDir, tempdir};

    use super::*;

    /// Helper function to create a temporary directory with a CODEOWNERS file
    fn setup_test_dir(codeowners_content: &str) -> (TempDir, PathBuf) {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let codeowners_path = temp_dir.path().join("CODEOWNERS");

        let mut file = File::create(&codeowners_path).expect("Failed to create CODEOWNERS file");
        file.write_all(codeowners_content.as_bytes()).expect("Failed to write to CODEOWNERS file");

        // Create some directories for testing
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src directory");

        let docs_dir = temp_dir.path().join("docs");
        fs::create_dir(&docs_dir).expect("Failed to create docs directory");

        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).expect("Failed to create tests directory");

        // Create some files for testing
        File::create(src_dir.join("main.rs")).expect("Failed to create main.rs");
        File::create(docs_dir.join("README.md")).expect("Failed to create README.md");

        (temp_dir, codeowners_path)
    }

    #[test]
    fn test_parse_from_file() {
        let codeowners_content = r#"
# This is a comment
/src/ @dev-team
/docs/ @docs-team @dev-team
/tests/ 
        "#;

        let (temp_dir, codeowners_path) = setup_test_dir(codeowners_content);
        let root = temp_dir.path().to_path_buf();

        let code_owners = CodeOwners::parse_from_file(&codeowners_path, &root).unwrap();

        // Check if teams exist
        assert!(code_owners.has_team("@dev-team"));
        assert!(code_owners.has_team("@docs-team"));
        assert!(!code_owners.has_team("@non-existent-team"));

        // Check patterns for teams
        let dev_team_patterns = code_owners.get_patterns_for_team("@dev-team");
        assert_eq!(dev_team_patterns.len(), 2);

        let docs_team_patterns = code_owners.get_patterns_for_team("@docs-team");
        assert_eq!(docs_team_patterns.len(), 1);

        // Check ignored patterns
        let ignored_patterns = code_owners.get_ignored_patterns();
        assert_eq!(ignored_patterns.len(), 1);
    }

    #[test]
    fn test_is_owned_by() {
        let codeowners_content = r#"
/src/ @dev-team
/docs/ @docs-team @dev-team
/tests/
        "#;

        let (temp_dir, codeowners_path) = setup_test_dir(codeowners_content);
        let root = temp_dir.path().to_path_buf();

        let code_owners = CodeOwners::parse_from_file(&codeowners_path, &root).unwrap();

        // Test file ownership
        assert!(code_owners.is_owned_by(&root.join("src/main.rs"), "@dev-team"));
        assert!(!code_owners.is_owned_by(&root.join("src/main.rs"), "@docs-team"));

        assert!(code_owners.is_owned_by(&root.join("docs/README.md"), "@docs-team"));
        assert!(code_owners.is_owned_by(&root.join("docs/README.md"), "@dev-team"));

        // Test directory ownership
        assert!(code_owners.is_owned_by(&root.join("src/"), "@dev-team"));
        assert!(!code_owners.is_owned_by(&root.join("src/"), "@docs-team"));
    }

    #[test]
    fn test_is_owned() {
        let codeowners_content = r#"
/src/ @dev-team
/docs/ @docs-team
/tests/
        "#;

        let (temp_dir, codeowners_path) = setup_test_dir(codeowners_content);
        let root = temp_dir.path().to_path_buf();

        let code_owners = CodeOwners::parse_from_file(&codeowners_path, &root).unwrap();

        // Test if files are owned by any team
        assert!(code_owners.is_owned(&root.join("src/main.rs")));
        assert!(code_owners.is_owned(&root.join("docs/README.md")));

        // Tests directory is ignored, so it should not be owned
        assert!(!code_owners.is_owned(&root.join("tests")));
    }

    #[test]
    fn test_lookup() {
        let codeowners_content = r#"
/src/ @dev-team
/docs/ @docs-team @dev-team
/tests/
        "#;

        let (temp_dir, codeowners_path) = setup_test_dir(codeowners_content);
        let root = temp_dir.path().to_path_buf();

        let code_owners = CodeOwners::parse_from_file(&codeowners_path, &root).unwrap();

        // Test lookup for files
        let src_owners = code_owners.lookup(&root.join("src/main.rs"));
        assert_eq!(src_owners.len(), 1);
        assert!(src_owners.contains(&"@dev-team".to_string()));

        let docs_owners = code_owners.lookup(&root.join("docs/README.md"));
        assert_eq!(docs_owners.len(), 2);
        assert!(docs_owners.contains(&"@docs-team".to_string()));
        assert!(docs_owners.contains(&"@dev-team".to_string()));

        // Test lookup for directories
        let src_dir_owners = code_owners.lookup(&root.join("src/"));
        assert_eq!(src_dir_owners.len(), 1);
        assert!(src_dir_owners.contains(&"@dev-team".to_string()));
    }

    #[test]
    fn test_complex_patterns() {
        let codeowners_content = r#"
# Root files
/**/*.md @docs-team

# Source code
/src/*.rs @dev-team
/src/api/ @api-team
/src/ui/ @ui-team

# Documentation with multiple owners
/docs/ @docs-team @dev-team

# External libraries
/lib/ @dev-team @sec-team

# Tests with no explicit owner
/tests/
        "#;

        let (temp_dir, codeowners_path) = setup_test_dir(codeowners_content);
        let root = temp_dir.path().to_path_buf();

        // Create additional directories and files for testing
        let src_api_dir = root.join("src/api");
        let src_ui_dir = root.join("src/ui");
        let lib_dir = root.join("lib");

        fs::create_dir_all(&src_api_dir).expect("Failed to create src/api directory");
        fs::create_dir_all(&src_ui_dir).expect("Failed to create src/ui directory");
        fs::create_dir_all(&lib_dir).expect("Failed to create lib directory");

        File::create(root.join("README.md")).expect("Failed to create README.md");
        File::create(src_api_dir.join("api.rs")).expect("Failed to create api.rs");
        File::create(src_ui_dir.join("ui.rs")).expect("Failed to create ui.rs");
        File::create(lib_dir.join("external.rs")).expect("Failed to create external.rs");

        let code_owners = CodeOwners::parse_from_file(&codeowners_path, &root).unwrap();

        // Test root markdown files
        let readme_owners = code_owners.lookup(&root.join("README.md"));
        assert_eq!(readme_owners.len(), 1);
        assert!(readme_owners.contains(&"@docs-team".to_string()));

        // Test src files
        let main_owners = code_owners.lookup(&root.join("src/main.rs"));
        assert_eq!(main_owners.len(), 1);
        assert!(main_owners.contains(&"@dev-team".to_string()));

        // Test api files
        let api_owners = code_owners.lookup(&root.join("src/api/api.rs"));
        assert_eq!(api_owners.len(), 2);
        assert!(api_owners.contains(&"@api-team".to_string()));
        assert!(api_owners.contains(&"@dev-team".to_string()));

        // Test ui files
        let ui_owners = code_owners.lookup(&root.join("src/ui/ui.rs"));
        assert_eq!(ui_owners.len(), 2);
        assert!(ui_owners.contains(&"@ui-team".to_string()));
        assert!(ui_owners.contains(&"@dev-team".to_string()));

        // Test lib files
        let lib_owners = code_owners.lookup(&root.join("lib/external.rs"));
        assert_eq!(lib_owners.len(), 2);
        assert!(lib_owners.contains(&"@sec-team".to_string()));
        assert!(lib_owners.contains(&"@dev-team".to_string()));
    }
}
