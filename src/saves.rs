use crate::error;

use error::BirdError;
use serde::{Deserialize, Serialize};

use std::{ffi::OsStr, fs::{self, File}, io::{BufReader, BufWriter}, os::unix::fs::MetadataExt, path::{Path, PathBuf}, process::Command, time::SystemTime};

#[derive(Clone)]
pub struct SaveFolder {
    pub name: String,
    pub path: PathBuf,
    pub is_current: bool
}

const CURRENT_SAVEGAMES_DIR: &str = "save games";
const EU4_SAVE_EXTENSION: &str = "eu4";

pub fn get_eu4_base_folder() -> Result<PathBuf, BirdError> {
    let base = if cfg!(target_os = "windows") {
        dirs::document_dir()
    } else {
        dirs::data_dir()
    }
    .ok_or(BirdError::UserDirNotFound)?;
    
    Ok(base.join("Paradox Interactive").join("Europa Universalis IV"))
}

pub fn get_current_save_folder() -> Result<Option<SaveFolder>, BirdError> {
    let eu4_base_dir = get_eu4_base_folder()?;
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
    let eu4_base_dir = get_eu4_base_folder()?;
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

pub fn backup_save_folder(name: Option<String>) -> Result<Option<PathBuf>, BirdError> {
    let eu4_base_dir = get_eu4_base_folder()?;
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

pub fn get_save_folder_by_index(index: usize) -> Result<SaveFolder, BirdError> {
    let save_folders = list_save_folders()?;
    
    save_folders
    .into_iter()
    .nth(index)
    .ok_or(BirdError::SaveFolderNotFound)
}

pub fn get_save_folder_by_name(name: String) -> Result<SaveFolder, BirdError> {
    let save_folders = list_save_folders()?;
    
    save_folders
    .into_iter()
    .find(|sf| sf.name == name)
    .ok_or(BirdError::SaveFolderNotFound)
}

pub fn restore_save_folder(save_game: SaveFolder, backup: bool) -> Result<(), BirdError> {
    // no need to restore if save games to restore are current
    if save_game.name == CURRENT_SAVEGAMES_DIR {
        return Ok(())
    }
    
    if backup {
        backup_save_folder(None)?;
    }
    
    // clear current save games
    let eu4_base_dir = get_eu4_base_folder()?;
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

#[derive(Deserialize, Serialize, Debug)]
struct Eu4Version {
    first: u8,
    second: u8,
    third: u8
}

#[derive(Deserialize, Serialize, Debug)]
struct Eu4Save {
    date: String,
    displayed_country_name: String,
    savegame_version: Eu4Version,
    is_ironman: bool,
    campaign_id: String
}

#[derive(Deserialize, Serialize, Debug)]
struct Eu4SaveMetadata {
    save_data: Eu4Save,
    file_size: u64,
    modified_at: SystemTime
}

pub fn read_save_data(save_folder: SaveFolder) -> Result<(), BirdError> {
    let save_files = get_save_files_in_folder(save_folder)?;

    for save_file in save_files {
        if save_file.extension() != Some(OsStr::new(EU4_SAVE_EXTENSION)) {
            continue;
        }

        let save_metadata_file = get_save_file_metadata(&save_file)?;
        match save_metadata_file {
            Some(save_metadata_file) => {
                println!("success: player country: {}, date: {}, version {}.{}.{}", save_metadata_file.save_data.displayed_country_name, save_metadata_file.save_data.date, save_metadata_file.save_data.savegame_version.first, save_metadata_file.save_data.savegame_version.second, save_metadata_file.save_data.savegame_version.third);
                println!("success: ironman: {}, campaign_id: {}", save_metadata_file.save_data.is_ironman, save_metadata_file.save_data.campaign_id);
                println!();
                continue;
            },
            _ => {}
        }        

        let file_path = save_file.to_str();
        println!("Reading file: {}", save_file.display());

        match file_path {
            Some(path) => {
                // TODO: use rakaly json and read props, save to .bird.save
                let command = Command::new("rakaly")
                    .arg("json")
                    .arg(path)
                    .output()
                    .expect("failed to execute process");

                let json: Eu4Save = serde_json::from_slice(&command.stdout)
                    .map_err(|e| { eprintln!("json slice error: {}", e);BirdError::SavaGameDataReadFailed } )?;

                println!("success: player country: {}, date: {}, version {}.{}.{}", json.displayed_country_name, json.date, json.savegame_version.first, json.savegame_version.second, json.savegame_version.third);
                println!("success: ironman: {}, campaign_id: {}", json.is_ironman, json.campaign_id);
                println!();

                let file_metadata = fs::metadata(&save_file).map_err(|_e| BirdError::ReadFileFailed)?;
                let file_modified_at = file_metadata.modified().map_err(|_e| BirdError::ReadFileFailed)?;
                let metadata = Eu4SaveMetadata {
                    file_size: file_metadata.size(),
                    modified_at: file_modified_at,
                    save_data: json
                };

                write_save_file_metadata(save_file, metadata)?
            },
            _ => {}
        }
    }
    
    Ok(())
}

pub fn get_save_file_metadata(save_file: &PathBuf) -> Result<Option<Eu4SaveMetadata>, BirdError> {
    let metadata_prefix = ".bird.";
    let Some(parent_dir) = save_file.parent() else { return Err(BirdError::InvalidPath(save_file.to_path_buf())) };
    let Some(file_name) = save_file.file_name() else { return Err(BirdError::InvalidPath(save_file.to_path_buf())) };
    let Some(file_name_str) = file_name.to_str() else { return Err(BirdError::InvalidPath(save_file.to_path_buf())) };
    
    let metadata_file_name = &format!("{}{}", metadata_prefix, file_name_str);
    let save_metadata_file = parent_dir.join(Path::new(metadata_file_name));

    if save_metadata_file.exists() {
        let file = File::open(save_metadata_file)?;
        let reader = BufReader::new(file);

        let metadata: Eu4SaveMetadata = serde_json::from_reader(reader).map_err(|_e| BirdError::ReadFileFailed)?;
        let file_metadata = fs::metadata(save_file).map_err(|_e| BirdError::ReadFileFailed)?;

        let file_modified_at = file_metadata.modified().map_err(|_e| BirdError::ReadFileFailed)?;
        if file_modified_at != metadata.modified_at || file_metadata.size() != metadata.file_size {
            return Ok(None)
        }
        return Ok(Some(metadata))
    }
    
    Ok(None)
}

pub fn write_save_file_metadata(save_file: PathBuf, metadata: Eu4SaveMetadata) -> Result<(), BirdError> {
    let metadata_prefix = ".bird.";
    let Some(parent_dir) = save_file.parent() else { return Err(BirdError::InvalidPath(save_file)) };
    let Some(file_name) = save_file.file_name() else { return Err(BirdError::InvalidPath(save_file)) };
    let Some(file_name_str) = file_name.to_str() else { return Err(BirdError::InvalidPath(save_file)) };
    
    let metadata_file_name = &format!("{}{}", metadata_prefix, file_name_str);
    let save_metadata_file = parent_dir.join(Path::new(metadata_file_name));

    let file = File::create(save_metadata_file)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &metadata).map_err(|e| BirdError::ReadFileFailed)?;
    
    Ok(())
}