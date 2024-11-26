pub struct Settings {
    /// Whether we should or shouldn't look at files that are within the
    /// CODEOWNERS.
    pub respect_gitignore: bool,

    /// Path to the actual CODEOWNERS file.
    pub codeowners: PathBuf,

    /// The team to use as a filter for the path.
    pub team: String,

    /// The pattern that is being searched for.
    pub pattern: String,
}

impl Settings {
    pub fn new(
        respect_gitignore: bool,
        codeowners: PathBuf,
        team: String,
        pattern: String,
    ) -> Self {
        Settings {
            respect_gitignore,
            codeowners,
            team,
            pattern,
        }
    }
}
}
