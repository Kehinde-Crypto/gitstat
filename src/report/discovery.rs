use std::path::{Path, PathBuf};
use walkdir::{Walkdir};

pub fn find_repos(root: &Path) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut it = WalkDir::new(root).into_iter();

    while let Some(entry) = it.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
            it.skip_current_dir();
        }
    }

    repos
}