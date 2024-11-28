use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

/// Convert any path to an absolute path (based on the current working
/// directory).
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize() {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

/// Convert any path to an absolute path (based on the specified project root).
pub fn normalize_path_to<P: AsRef<Path>, R: AsRef<Path>>(path: P, project_root: R) -> PathBuf {
    let path = path.as_ref();
    if let Ok(path) = path.absolutize_from(project_root.as_ref()) {
        return path.to_path_buf();
    }
    path.to_path_buf()
}

/// Compute the common root for a list of files.
pub fn common_root(paths: &[PathBuf]) -> PathBuf {
    let mut common_root = paths[0].clone();
    for path in paths.iter().skip(1) {
        common_root = common_root
            .components()
            .zip(path.components())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a)
            .collect();
    }
    common_root
}
