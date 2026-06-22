mod error;

use clap::{Parser, Subcommand};
use std::{ffi::OsStr, fs, path::PathBuf};
use error::BirdError;

/// Simple CLI tool to manage Europa Universalis IV save games
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    List,
    Backup,
}

fn eu4_base_dir() -> Result<PathBuf, BirdError> {
    let base = if cfg!(target_os = "windows") {
        dirs::document_dir()
    } else {
        dirs::data_dir()
    }
    .ok_or(BirdError::UserDirNotFound)?;

    Ok(base.join("Paradox Interactive")
        .join("Europa Universalis IV"))
}

fn main() -> Result<(), BirdError> {
    let cli = Cli::parse();

    // TODO: Make these configurable, this depends on the OS
    let current_savegames_dir_name = "save games";
    let eu4_save_extension = "eu4";
    let eu4_base_dir = eu4_base_dir()?;

    match cli.cmd {
        Commands::List => {
            if !eu4_base_dir.exists() {
                return Err(BirdError::DirNotFound(eu4_base_dir))
            }

            let all_dirs = fs::read_dir(&eu4_base_dir)?;

            println!("Save games directory: {:?}", eu4_base_dir);
            println!();
            println!("Available save games folders:");

            for dir_entry in all_dirs {
                let Ok(dir_entry) = dir_entry else { continue; };
                let Ok(file_type) = dir_entry.file_type() else { continue; };
                let true = file_type.is_dir() else { continue; };
                let dir_path = dir_entry.path();

                let mut dir_files = dir_path.read_dir()?;

                let dir_contains_saves = dir_files.any(|file| {
                    let Ok(entry) = file else { return false; };
                    let Ok(ft) = entry.file_type() else { return false; };
                    ft.is_file() && entry.path().extension() == Some(OsStr::new(eu4_save_extension))
                });

                let dir_name = dir_path.file_name().ok_or(BirdError::InvalidUtf8InPath(dir_path.clone()))?;
                if dir_contains_saves {
                    let name = dir_name.to_str().ok_or(BirdError::InvalidUtf8InPath(dir_path.clone()))?;
                    println!("{}", name);
                }
            }

            Ok(())
        }
        Commands::Backup => {
            if !eu4_base_dir.exists() {
                return Err(BirdError::DirNotFound(eu4_base_dir))
            }

            let current_savegames_dir = eu4_base_dir.join(current_savegames_dir_name);
            if !current_savegames_dir.exists() {
                println!("No save games found, skipping backup.");
                return Ok(());
            }
            
            let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
            let backup_dest_dir = eu4_base_dir.join(format!("save games {timestamp}"));
            fs::create_dir(backup_dest_dir.as_path())?;

            let copy_options = fs_extra::dir::CopyOptions::new().overwrite(true);
            fs_extra::dir::copy(&current_savegames_dir, &backup_dest_dir, &copy_options)
                .map_err(|e| BirdError::BackupFailed(e.to_string()))?;

            println!("Backup created: {}", backup_dest_dir.display());

            Ok(())
        }
    }
}
