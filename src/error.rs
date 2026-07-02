use thiserror::Error;

#[derive(Debug, Error)]
pub enum BirdError {
    #[error("Could not find user directory")]
    UserDirNotFound,

    #[error("Path is invalid: {0:?}")]
    InvalidPath(std::path::PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Directory not found: {0}")]
    DirNotFound(std::path::PathBuf),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Restore failed: {0}")]
    RestoreFailed(String),

    #[error("Save folder not found")]
    SaveFolderNotFound,

    #[error("Cannot read file")]
    ReadFileFailed
}