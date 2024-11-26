//! Utilities for showing the version of the tool.

use std::fmt;

pub(crate) struct CommitInfo {
    short_commit_hash: String,
    commit_date: String,
}

pub(crate) struct VersionInfo {
    /// Teamsearch version.
    version: String,

    /// Information about the git commit we may have been built from.
    ///
    /// `None` if not built from a git repo or if retrieval failed.
    commit_info: Option<CommitInfo>,
}

impl fmt::Display for VersionInfo {
    /// Formatted version information: "<version>[+<commits>] (<commit> <date>)"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;

        if let Some(ref ci) = self.commit_info {
            write!(f, " ({} {})", ci.short_commit_hash, ci.commit_date)?;
        }

        Ok(())
    }
}

pub fn version() -> VersionInfo {
    // Environment variables are only read at compile-time
    macro_rules! option_env_str {
        ($name:expr) => {
            option_env!($name).map(|s| s.to_string())
        };
    }

    // This version is pulled from Cargo.toml and set by Cargo
    let version = option_env_str!("CARGO_PKG_VERSION").unwrap();

    // Commit info is pulled from git and set by `build.rs`
    let commit_info = option_env_str!("TEAMSEARCH_COMMIT_HASH").map(|_| CommitInfo {
        short_commit_hash: option_env_str!("TEAMSEARCH_COMMIT_SHORT_HASH").unwrap(),
        commit_date: option_env_str!("TEAMSEARCH_COMMIT_DATE").unwrap(),
    });

    VersionInfo { version, commit_info }
}
