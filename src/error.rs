use thiserror::Error;

#[derive(Debug, Error)]
pub enum BirdError {
    #[error("could not find user directory")]
    UserDirNotFound,

    #[error("path contains invalid UTF-8: {0:?}")]
    InvalidUtf8InPath(std::path::PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Directory not found: {0}")]
    DirNotFound(std::path::PathBuf),

    #[error("backup failed: {0}")]
    BackupFailed(String),
}