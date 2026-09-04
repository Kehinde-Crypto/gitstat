use std::path::{Path, PathBuf};
use walkdir::{WalkDir};

pub fn find_repos(root: &Path , max_depth: usize) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut it = WalkDir::new(root).max_depth(max_depth).into_iter();

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