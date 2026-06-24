use crate::error;

use error::BirdError;
use std::{ffi::OsStr, fs, path::PathBuf};

#[derive(Clone)]
pub struct SaveFolder {
    pub name: String,
    pub path: PathBuf
}

const CURRENT_SAVEGAMES_DIR: &str = "save games";
const EU4_SAVE_EXTENSION: &str = "eu4";

pub fn get_eu4_base_dir() -> Result<PathBuf, BirdError> {
    let base = if cfg!(target_os = "windows") {
        dirs::document_dir()
    } else {
        dirs::data_dir()
    }
    .ok_or(BirdError::UserDirNotFound)?;
    
    Ok(base.join("Paradox Interactive").join("Europa Universalis IV"))
}

pub fn list_save_folders() -> Result<Vec<SaveFolder>, BirdError> {
    let eu4_base_dir = get_eu4_base_dir()?;
    if !eu4_base_dir.exists() {
        return Err(BirdError::DirNotFound(eu4_base_dir))
    }
    
    let mut save_games_folders = Vec::new();
    let all_dirs = fs::read_dir(&eu4_base_dir)?;
    
    for dir_entry in all_dirs {
        let Ok(dir_entry) = dir_entry else { continue; };
        let Ok(file_type) = dir_entry.file_type() else { continue; };
        let true = file_type.is_dir() else { continue; };
        let dir_path = dir_entry.path();
        
        let mut dir_files = dir_path.read_dir()?;
        
        let dir_contains_saves = dir_files.any(|file| {
            let Ok(entry) = file else { return false; };
            let Ok(ft) = entry.file_type() else { return false; };
            ft.is_file() && entry.path().extension() == Some(OsStr::new(EU4_SAVE_EXTENSION))
        });
        
        if dir_contains_saves {
            let dir_name = dir_path.file_name().ok_or_else(|| BirdError::InvalidPath(dir_path.clone()))?;
            let name = dir_name.to_str().ok_or_else(|| BirdError::InvalidPath(dir_path.clone()))?;
            save_games_folders.push(SaveFolder {
                name: name.to_string(),
                path: dir_path
            });
        }
    }
    
    Ok(save_games_folders)
}

pub fn backup_saves() -> Result<Option<PathBuf>, BirdError> {
    let eu4_base_dir = get_eu4_base_dir()?;
    if !eu4_base_dir.exists() {
        return Err(BirdError::DirNotFound(eu4_base_dir))
    }

    let current_savegames_dir = eu4_base_dir.join(CURRENT_SAVEGAMES_DIR);
    if !current_savegames_dir.exists() {
        return Ok(None);
    }
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let backup_dest_dir = eu4_base_dir.join(format!("{CURRENT_SAVEGAMES_DIR} {timestamp}"));
    fs::create_dir(backup_dest_dir.as_path())?;
    
    let copy_options = fs_extra::dir::CopyOptions::new().overwrite(true).copy_inside(true);
    fs_extra::dir::copy(&current_savegames_dir, &backup_dest_dir, &copy_options)
        .map_err(|e| BirdError::BackupFailed(e.to_string()))?;
        
    Ok(Some(backup_dest_dir))
}

pub fn get_save_games_by_index(index: usize) -> Result<SaveFolder, BirdError> {
    let save_folders = list_save_folders()?;

    save_folders
        .into_iter()
        .nth(index)
        .ok_or(BirdError::SaveFolderNotFound)
}

pub fn get_save_games_by_name(name: String) -> Result<SaveFolder, BirdError> {
    let save_folders = list_save_folders()?;

    save_folders
        .into_iter()
        .find(|sf| sf.name == name)
        .ok_or(BirdError::SaveFolderNotFound)
}

pub fn restore_save(save_game: SaveFolder, backup: bool) -> Result<(), BirdError> {
    // no need to restore if save games to restore are current
    if save_game.name == CURRENT_SAVEGAMES_DIR {
        return Ok(())
    }

    if backup {
        backup_saves()?;
    }

    // clear current save games
    let eu4_base_dir = get_eu4_base_dir()?;
    if !eu4_base_dir.exists() {
        return Err(BirdError::DirNotFound(eu4_base_dir))
    }

    let current_savegames_dir = eu4_base_dir.join(CURRENT_SAVEGAMES_DIR);
    if !current_savegames_dir.exists() {
        fs::create_dir(CURRENT_SAVEGAMES_DIR)?;
    }

    for entry in fs::read_dir(&current_savegames_dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    // copy contents
    let copy_options = fs_extra::dir::CopyOptions::new().overwrite(true).copy_inside(true);
    fs_extra::dir::copy(&save_game.path, &current_savegames_dir, &copy_options)
        .map_err(|e| BirdError::RestoreFailed(e.to_string()))?;

    Ok(())
}