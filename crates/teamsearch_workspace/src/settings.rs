//! Defines all of the settings that a [super::Workspace] can hold.

use std::{fmt, ops::Deref, path::PathBuf, str::FromStr};

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum FilePattern {
    Builtin(&'static str),
    User(String),
}

impl FilePattern {
    pub fn add_to(self, builder: &mut GlobSetBuilder) -> Result<()> {
        match self {
            FilePattern::Builtin(pattern) => {
                builder.add(Glob::from_str(pattern)?);
            }
            FilePattern::User(pattern) => {
                builder.add(Glob::from_str(&pattern)?);
            }
        }
        Ok(())
    }
}

impl fmt::Display for FilePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                Self::Builtin(pattern) => pattern,
                Self::User(pattern) => pattern.as_str(),
            }
        )
    }
}

/// Typical directories that we don't care about.
pub(crate) static EXCLUDE: &[FilePattern] = &[
    FilePattern::Builtin(".bzr"),
    FilePattern::Builtin(".direnv"),
    FilePattern::Builtin(".eggs"),
    FilePattern::Builtin(".git"),
    FilePattern::Builtin(".git-rewrite"),
    FilePattern::Builtin(".hg"),
    FilePattern::Builtin(".ipynb_checkpoints"),
    FilePattern::Builtin(".mypy_cache"),
    FilePattern::Builtin(".nox"),
    FilePattern::Builtin(".pants.d"),
    FilePattern::Builtin(".pyenv"),
    FilePattern::Builtin(".pytest_cache"),
    FilePattern::Builtin(".pytype"),
    FilePattern::Builtin(".ruff_cache"),
    FilePattern::Builtin(".svn"),
    FilePattern::Builtin(".tox"),
    FilePattern::Builtin(".venv"),
    FilePattern::Builtin(".vscode"),
    FilePattern::Builtin("__pypackages__"),
    FilePattern::Builtin("__pycache__"),
    FilePattern::Builtin("_build"),
    FilePattern::Builtin("buck-out"),
    FilePattern::Builtin("dist"),
    FilePattern::Builtin("node_modules"),
    FilePattern::Builtin("site-packages"),
    FilePattern::Builtin("venv"),
];

/// A list of file patterns that we should look at in particular by default.
pub(crate) static INCLUDE: &[FilePattern] = &[FilePattern::Builtin("examples/notes/**")];

#[derive(Debug, Clone, Default)]
pub struct FilePatternSet {
    /// The actual set of globs that are used to match files.
    set: GlobSet,

    /// The internal representation of the file patterns.
    _set_internals: Vec<FilePattern>,
}

impl FilePatternSet {
    pub fn try_from_iter<I>(patterns: I) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = FilePattern>,
    {
        let mut builder = GlobSetBuilder::new();

        let mut _set_internals = vec![];

        for pattern in patterns {
            _set_internals.push(pattern.clone());
            pattern.add_to(&mut builder)?;
        }

        let set = builder.build()?;

        Ok(FilePatternSet { set, _set_internals })
    }

    pub fn into_file_patterns_iter(self) -> impl Iterator<Item = FilePattern> {
        self._set_internals.into_iter()
    }

    pub fn extend<I>(self, patterns: I) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = FilePattern>,
    {
        Self::try_from_iter(patterns.into_iter().chain(self._set_internals))
    }
}

impl Deref for FilePatternSet {
    type Target = GlobSet;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

/// The settings for the file resolver.
#[derive(Default)]
pub struct FileResolverSettings {
    /// Files that are explicitly included in the [super::Workspace].
    pub include: FilePatternSet,

    /// Files that are explicitly excluded from the [super::Workspace].
    pub exclude: FilePatternSet,

    /// Any user extensions to the exclusion patterns.
    pub user_exclude: FilePatternSet,

    /// Whether to enforce file exclusions.
    pub force_exclude: bool,
}

impl FileResolverSettings {
    pub fn new() -> Self {
        FileResolverSettings {
            include: FilePatternSet::try_from_iter(INCLUDE.iter().cloned()).unwrap(),
            exclude: FilePatternSet::try_from_iter(EXCLUDE.iter().cloned()).unwrap(),
            user_exclude: FilePatternSet::default(),
            force_exclude: false,
        }
    }
}

pub struct Settings {
    /// Whether we should or shouldn't look at files that are within the
    /// CODEOWNERS.
    pub respect_gitignore: bool,

    /// Path to the actual CODEOWNERS file.
    pub codeowners: PathBuf,

    /// Settings to do with file exclusions/inclusions.
    pub file_resolver: FileResolverSettings,
}

impl Settings {
    pub fn new(respect_gitignore: bool, codeowners: PathBuf) -> Self {
        Settings { respect_gitignore, codeowners, file_resolver: FileResolverSettings::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_pattern_set() {
        let patterns = vec![FilePattern::User("notes/**".into())];
        let set = FilePatternSet::try_from_iter(patterns).unwrap();

        assert_eq!(set.len(), 1);
        assert!(set.is_match("examples/notes/index.html"));
        assert!(set.is_match("examples/notes/index.js"));
        assert!(set.is_match("examples/notes/sub/index.js"));
    }
}
