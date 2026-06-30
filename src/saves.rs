use crate::error;

use error::BirdError;
use eu4save::{EnvTokens, Eu4File, query::Query};
use std::{ffi::OsStr, fs::{self}, path::PathBuf, process::Command};

#[derive(Clone)]
pub struct SaveFolder {
    pub name: String,
    pub path: PathBuf,
    pub is_current: bool
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

pub fn get_current_save_games() -> Result<Option<SaveFolder>, BirdError> {
    let eu4_base_dir = get_eu4_base_dir()?;
    if !eu4_base_dir.exists() {
        return Err(BirdError::DirNotFound(eu4_base_dir))
    }

    let current_save_games_dir = eu4_base_dir.join(CURRENT_SAVEGAMES_DIR);
    if !current_save_games_dir.exists() {
        return Ok(None);
    }

   Ok(Some(SaveFolder {
        is_current: true,
        name: CURRENT_SAVEGAMES_DIR.to_string(),
        path: current_save_games_dir
    }))
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
                is_current: name == CURRENT_SAVEGAMES_DIR,
                path: dir_path
            });
        }
    }
    
    save_games_folders.sort_by_key(|sf| sf.is_current);
    save_games_folders.reverse();
    Ok(save_games_folders)
}

pub fn backup_saves(name: Option<String>) -> Result<Option<PathBuf>, BirdError> {
    let eu4_base_dir = get_eu4_base_dir()?;
    if !eu4_base_dir.exists() {
        return Err(BirdError::DirNotFound(eu4_base_dir))
    }
    
    let current_savegames_dir = eu4_base_dir.join(CURRENT_SAVEGAMES_DIR);
    if !current_savegames_dir.exists() {
        return Ok(None);
    }
    
    let backup_dest_dir = match name {
        Some(name) => eu4_base_dir.join(format!("{CURRENT_SAVEGAMES_DIR} - {name}")),
        None => {
            let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
            eu4_base_dir.join(format!("{CURRENT_SAVEGAMES_DIR} - {timestamp}"))
        }
    };

    if backup_dest_dir.exists() {
        return Err(BirdError::BackupFailed(format!("'{}' directory already exists", backup_dest_dir.display())))
    }
    
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
        backup_saves(None)?;
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

pub fn get_save_files_in_folder(save_folder: SaveFolder) -> Result<Vec<PathBuf>, BirdError> {
    let dir_files = save_folder.path.read_dir()?;
    
    let save_files: Vec<PathBuf> = dir_files
        .filter_map(|file| {
            let entry = file.ok()?;
            let ft = entry.file_type().ok()?;
            (ft.is_file() && entry.path().extension() == Some(OsStr::new(EU4_SAVE_EXTENSION)))
                .then_some(entry.path())
        })
        .collect();
    
    Ok(save_files)
}

pub fn read_save_data(save_folder: SaveFolder) -> Result<(), BirdError> {
    let save_files = get_save_files_in_folder(save_folder)?;

    for save_file in save_files {
        let data = std::fs::read(&save_file)?;
        let melted_file_data : Vec<u8>;
        println!("Got file {} data", save_file.display());
        let mut file = Eu4File::from_slice(&data).map_err(|e| { eprintln!("from_slice error: {}", e); BirdError::SavaGameDataReadFailed })?;
        println!("Got eu4 file");
        let save_encoding = file.encoding();

        println!("{}", save_encoding.as_str());
        match save_encoding {
            eu4save::Encoding::Binary | eu4save::Encoding::BinaryZip => {
                let file_path = save_file.to_str();
                match file_path {
                    Some(path) => {
                        let command = Command::new("rakaly")
                            .arg("melt")
                            .arg("-c")
                            .arg(path)
                            .output()
                            .expect("failed to execute process");

                        melted_file_data = command.stdout;
                        println!("Got melted file data with count {}", melted_file_data.iter().count());
                        //let output = String::from_utf8_lossy(&melted_file_data);
                        //println!("{}", output);
                        file = Eu4File::from_slice(&melted_file_data).map_err(|e| { eprintln!("from_slice error: {}", e); BirdError::SavaGameDataReadFailed })?;
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        println!("got eu4 file after");
        let save = file.deserializer().build_save(&EnvTokens).map_err(|e| { eprintln!("build_save error: {}", e); BirdError::SavaGameDataReadFailed })?;
        println!("Got save data");
        let players = Query::from_save(save).players();
        println!("Got players data");

        for player in players {
            println!("{} {}", player.name, player.tag.as_str());
        }
    }
    
    Ok(())
}  