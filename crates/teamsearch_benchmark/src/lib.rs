//! Repositories and patterns that the teamsearch benchmarks run on.
//!
//! Each [Repo] is written to a temporary directory and shaped like a monorepo:
//! services that teams own through a `.github/CODEOWNERS` file, some services
//! that no team owns, and directories that `teamsearch` skips, such as
//! `node_modules/` and those that `.gitignore` lists. They are generated rather
//! than copied from real projects, so that their size is fixed and every
//! benchmark sees the same files.

use std::{
    fmt::{self, Write},
    fs,
    path::PathBuf,
};

use teamsearch_matcher::Pattern;
use teamsearch_workspace::{codeowners::CodeOwners, settings::Settings};
use tempfile::TempDir;

/// The team that the benchmarks resolve the files of, as `teamsearch find
/// --team` does.
pub const TEAM: &str = "@org/team-0";

/// The modules in the source of each service.
const MODULES: usize = 4;

/// The files in each module of a service.
const MODULE_FILES: [&str; 4] = ["mod", "handler", "model", "routes"];

/// The functions in each source file.
const FUNCTIONS: usize = 8;

/// The size of a [Repo].
///
/// The benchmarks take a [Size] and write the [Repo] themselves: divan never
/// drops the arguments of a benchmark, so a [Repo] that was one would never be
/// removed.
#[derive(Clone, Copy)]
pub enum Size {
    /// 20 services, with about 450 files and a CODEOWNERS file of 70 rules.
    Small,

    /// 200 services, with about 4,500 files and a CODEOWNERS file of 650
    /// rules.
    Large,
}

impl Size {
    /// Every size of [Repo] that the benchmarks run on.
    pub const ALL: [Size; 2] = [Size::Small, Size::Large];

    /// The services in a [Repo] of this size.
    fn services(self) -> usize {
        match self {
            Size::Small => 20,
            Size::Large => 200,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Size::Small => "small",
            Size::Large => "large",
        })
    }
}

/// A repository that a benchmark parses the CODEOWNERS file of, or resolves,
/// looks up or searches the files of.
pub struct Repo {
    /// The size of the repository.
    pub size: Size,

    /// The root of the repository.
    pub root: PathBuf,

    /// The path of the repository's CODEOWNERS file.
    pub codeowners: PathBuf,

    /// Every file that `teamsearch orphans` looks at, i.e. all of them except
    /// hidden ones and those in directories that it skips.
    pub files: Vec<PathBuf>,

    /// The directory that holds the repository, which is removed when the
    /// [Repo] is dropped.
    _dir: TempDir,
}

impl Repo {
    /// Write a repository of the given `size` to a temporary directory.
    pub fn new(size: Size) -> Self {
        let dir = tempfile::tempdir().expect("failed to create a directory for the repository");
        let mut repo = RepoWriter::new(dir.path().to_path_buf());
        let services = size.services();
        let teams = services / 2;

        repo.rule("*.md", &["@org/docs"]);
        repo.rule("/docs/", &["@org/docs"]);
        repo.rule("**/migrations/**", &["@org/data"]);
        repo.rule("/libs/", &["@org/platform"]);
        repo.rule("/vendor/", &[]);

        for service in 0..services {
            let dir = format!("services/svc-{service}");

            // Every fifth service has no owners, so that its code is orphaned.
            if service % 5 != 4 {
                let team = format!("@org/team-{}", service / 2);
                let other_team = format!("@org/team-{}", (service / 2 + 1) % teams);

                repo.rule(&format!("/{dir}/"), &[&team]);
                repo.rule(&format!("/{dir}/src/main.rs"), &[&team, "@org/leads"]);
                repo.rule(&format!("/{dir}/src/module-{}/", service % MODULES), &[&other_team]);
                repo.rule(&format!("/{dir}/tests/"), &["@org/qa"]);
            }

            repo.file(&format!("{dir}/README.md"), &prose(service));
            repo.file(&format!("{dir}/src/main.rs"), &source(service));

            for module in 0..MODULES {
                for (index, file) in MODULE_FILES.iter().enumerate() {
                    let seed = service + module * MODULE_FILES.len() + index;
                    repo.file(&format!("{dir}/src/module-{module}/{file}.rs"), &source(seed));
                }
            }

            for test in 0..3 {
                repo.file(&format!("{dir}/tests/test_{test}.rs"), &source(service + test));
            }

            if service.is_multiple_of(3) {
                for migration in 0..2 {
                    repo.file(&format!("{dir}/migrations/{migration:04}_init.sql"), MIGRATION);
                }
            }
        }

        for index in 0..services / 4 {
            repo.file(&format!("docs/guide-{index}.md"), &prose(index));
            repo.file(&format!("libs/lib-{index}/src/lib.rs"), &source(index));

            // Directories that `teamsearch` skips: by default, and through
            // `.gitignore`.
            repo.skipped(&format!("node_modules/pkg-{index}/index.js"), "module.exports = {};\n");
            repo.skipped(&format!("target/debug/svc-{index}"), "\x7fELF");
        }

        for index in 0..services / 10 {
            repo.file(&format!("vendor/dep-{index}/src/lib.rs"), &source(index));
        }

        repo.file("README.md", &prose(services));

        // `teamsearch` skips hidden files, such as these. `.gitignore` only
        // applies within a git repository.
        repo.skipped(".gitignore", "/target/\n");
        repo.skipped(".git/HEAD", "ref: refs/heads/main\n");
        let codeowners = repo.skipped(".github/CODEOWNERS", &repo.codeowners);

        Repo { size, root: repo.root, codeowners, files: repo.files, _dir: dir }
    }

    /// Parse the CODEOWNERS file of the repository, as each command does first.
    pub fn parse_codeowners(&self) -> CodeOwners {
        CodeOwners::parse_from_file(&self.codeowners, &self.root)
            .expect("failed to parse the CODEOWNERS file")
    }

    /// The settings that `teamsearch find --team <team>` resolves files with:
    /// it includes the files that the team owns, and excludes those that the
    /// CODEOWNERS file leaves without owners.
    pub fn team_settings(&self, team: &str) -> Settings {
        let codeowners = self.parse_codeowners();
        let mut settings = Settings::new(true, self.codeowners.clone());

        let patterns = codeowners.get_patterns_for_team(team).to_vec();
        settings.file_resolver.include = settings.file_resolver.include.extend(patterns).unwrap();

        let ignored = codeowners.get_ignored_patterns().to_vec();
        settings.file_resolver.user_exclude =
            settings.file_resolver.user_exclude.extend(ignored).unwrap();

        settings
    }

    /// At most `count` of the files, spread evenly over the repository.
    pub fn sample(&self, count: usize) -> Vec<&PathBuf> {
        self.files.iter().step_by(self.files.len().div_ceil(count)).collect()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.size.fmt(f)
    }
}

/// Writes the files of a [Repo], and the rules of its CODEOWNERS file.
struct RepoWriter {
    root: PathBuf,
    files: Vec<PathBuf>,
    codeowners: String,
}

impl RepoWriter {
    fn new(root: PathBuf) -> Self {
        RepoWriter { root, files: vec![], codeowners: String::from("# Generated owners.\n\n") }
    }

    /// Add a rule to the CODEOWNERS file, giving `pattern` to `owners`.
    fn rule(&mut self, pattern: &str, owners: &[&str]) {
        writeln!(self.codeowners, "{pattern} {}", owners.join(" ")).unwrap();
    }

    /// Write a file that `teamsearch` looks at.
    fn file(&mut self, path: &str, contents: &str) {
        let path = self.skipped(path, contents);
        self.files.push(path);
    }

    /// Write a file that `teamsearch` skips, returning its path.
    fn skipped(&self, path: &str, contents: &str) -> PathBuf {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("failed to create a directory");
        fs::write(&path, contents).expect("failed to write a file");
        path
    }
}

/// A Rust source file of [FUNCTIONS] functions, around 2KB long. Some of the
/// functions have a `TODO` comment.
fn source(seed: usize) -> String {
    let mut source = String::from("use std::collections::HashMap;\n");

    for function in 0..FUNCTIONS {
        let todo = if (seed + function).is_multiple_of(5) {
            "    // TODO: tell an unknown code apart from one without a reason.\n"
        } else {
            ""
        };

        write!(
            source,
            "
/// Returns the reason for the status `code`, if it has one.
pub fn reason_{seed}_{function}(reasons: &HashMap<u16, String>, code: u16) -> Option<&str> {{
{todo}    let reason = reasons.get(&code)?;
    if reason.is_empty() {{
        return None;
    }}

    Some(reason.as_str())
}}
"
        )
        .unwrap();
    }

    source
}

/// A Markdown document of a few paragraphs.
fn prose(seed: usize) -> String {
    format!(
        "# Guide {seed}

This guide describes how the service handles a request, from the moment that
it arrives until the response is sent. Each request is routed to a handler,
which looks up the model that it needs and returns a status code.

## Status codes

A handler returns `200` when the request succeeds, `404` when the model that it
asks for doesn't exist and `500` when something else goes wrong. The reason for
each code is kept in a table, so that it can be changed without a release.

TODO: document the codes that the service retries on.
"
    )
}

/// A SQL migration.
const MIGRATION: &str = "CREATE TABLE IF NOT EXISTS reasons (
    code INTEGER PRIMARY KEY,
    reason TEXT NOT NULL
);
";

/// A pattern that a benchmark searches files for, as `teamsearch find
/// --pattern` does.
pub struct Search {
    /// The name that the benchmark reports the pattern under.
    pub name: &'static str,

    /// The pattern, which is a regular expression.
    pub pattern: &'static str,

    /// Whether the pattern matches regardless of case.
    pub case_insensitive: bool,
}

impl Search {
    fn new(name: &'static str, pattern: &'static str, case_insensitive: bool) -> Self {
        Search { name, pattern, case_insensitive }
    }

    /// Every pattern that the benchmarks search for.
    pub fn all() -> Vec<Search> {
        vec![
            // A word that a few lines of each file have.
            Self::new("literal", "TODO", false),
            Self::new("case_insensitive", "todo", true),
            // A regex that every function in a source file matches.
            Self::new("regex", r"fn \w+\(", false),
        ]
    }

    /// The [Pattern] that `teamsearch_matcher` searches for.
    pub fn pattern(&self) -> Pattern<'_> {
        Pattern::new(self.pattern, self.case_insensitive)
    }
}

impl fmt::Display for Search {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod tests {
    use std::slice;

    use teamsearch_matcher::search_file;
    use teamsearch_workspace::{resolver::find_files_in_paths, settings::FilePattern};

    use super::*;

    /// `teamsearch` should look at exactly the files in [Repo::files], and
    /// skip the directories that hold the rest.
    #[test]
    fn test_repos_resolve_their_files() {
        for repo in Size::ALL.map(Repo::new) {
            let mut settings = Settings::new(true, repo.codeowners.clone());
            settings.file_resolver.include =
                settings.file_resolver.include.extend([FilePattern::all()]).unwrap();

            let mut resolved: Vec<PathBuf> =
                find_files_in_paths(&repo.root, slice::from_ref(&repo.root), &settings)
                    .unwrap()
                    .into_iter()
                    .map(|file| file.unwrap().into_path())
                    .collect();

            let mut files = repo.files.clone();
            resolved.sort();
            files.sort();
            assert_eq!(resolved, files, "`{repo}` resolved other files than it lists");
        }
    }

    /// A repository whose files were all owned, or all orphaned, would only
    /// benchmark one outcome of `is_owned`.
    #[test]
    fn test_repos_have_owned_and_orphaned_files() {
        for repo in Size::ALL.map(Repo::new) {
            let codeowners = repo.parse_codeowners();
            let owned = repo.files.iter().filter(|path| codeowners.is_owned(path)).count();
            assert!(owned > 0, "`{repo}` has no owned files");
            assert!(owned < repo.files.len(), "`{repo}` has no orphaned files");
        }
    }

    /// The team that `find` is benchmarked with should own some of the files,
    /// but not all of them.
    #[test]
    fn test_team_owns_some_files() {
        for repo in Size::ALL.map(Repo::new) {
            let settings = repo.team_settings(TEAM);
            let resolved =
                find_files_in_paths(&repo.root, slice::from_ref(&repo.root), &settings).unwrap();
            assert!(!resolved.is_empty(), "`{TEAM}` owns no files in `{repo}`");
            assert!(resolved.len() < repo.files.len(), "`{TEAM}` owns every file in `{repo}`");
        }
    }

    /// A pattern that matched nothing would only benchmark the search failing.
    #[test]
    fn test_searches_find_matches() {
        let repo = Repo::new(Size::Small);

        for search in Search::all() {
            let matched = repo
                .sample(10)
                .into_iter()
                .any(|path| !search_file(search.pattern(), path.clone()).unwrap().is_empty());
            assert!(matched, "`{search}` matched nothing in `{repo}`");
        }
    }
}
