use anyhow::Result;
use teamsearch_workspace::{
    settings::Settings,
};

pub fn find(files: &[PathBuf], mut settings: Settings) -> Result<()> {
    let paths: Vec<PathBuf> = files.iter().map(fs::normalize_path).unique().collect();

    if paths.is_empty() {
        return Ok(());
    }

    Ok(())
}
}
