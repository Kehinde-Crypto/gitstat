use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitStatError {
    #[error("failed to spawn git process for path '{path}'")]
    GitSpawnFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("git command failed for path '{path}': {stderr}")]
    GitCommandFailed {
        path: PathBuf,
        stderr: String,
    },

    #[error("failed to walk directory tree at '{path}'")]
    WalkFailed { path: PathBuf },

    #[error("failed to parse output for path '{path}'")]
    ParseFailed { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, GitStatError>;