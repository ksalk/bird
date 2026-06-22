use thiserror::Error;

#[derive(Debug, Error)]
pub enum BirdError {
    #[error("could not find user directory")]
    UserDirNotFound,

    #[error("path is invalid: {0:?}")]
    InvalidPath(std::path::PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Directory not found: {0}")]
    DirNotFound(std::path::PathBuf),

    #[error("backup failed: {0}")]
    BackupFailed(String),
}