  use crate::error::{GitStatError, Result};
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};
  use std::process::Command;

  #[derive(Debug, Clone)]
  pub struct RepoStats {
      pub path: PathBuf,
      pub commit_count: usize,
      pub authors: HashMap<String, usize>,
  }

  pub fn scan_repo(path: &Path) -> Result<RepoStats>{

  let output = Command::new("git")
      .arg("-C")
      .arg(path)
      .arg("log")
      .arg("--pretty=format:%an")
      .output()
      .map_err(|source| GitStatError::GitSpawnFailed {
            path: path.to_path_buf(),
            source,
        })?;

  if  !output.status.success() {
    return Err(GitStatError::GitCommandFailed{
      path: path.to_path_buf(),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  let mut authors: HashMap<String, usize> = HashMap::new();
  let mut commit_count = 0;

  for line in stdout.lines(){
    let name = line.trim();
    if name.is_empty(){
      continue;
    }
    commit_count+= 1;
    *authors.entry(name.to_string()).or_insert(0) += 1;
  }

  Ok(RepoStats{
    path:path.to_path_buf(),
    commit_count,
    authors
  })
}
  