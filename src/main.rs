use clap::{Parser, Subcommand};
use std::{ffi::OsStr, fs, path::PathBuf};

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

fn eu4_base_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        dirs::document_dir()
    } else {
        dirs::data_dir()
    }
    .expect("Could not find user directory");

    base.join("Paradox Interactive")
        .join("Europa Universalis IV")
}

fn main() {
    let cli = Cli::parse();

    // TODO: Make these configurable, this depends on the OS
    let current_savegames_dir_name = "save games";
    let eu4_save_extension = "eu4";
    let eu4_base_dir = eu4_base_dir();

    match cli.cmd {
        Commands::List => {
            let all_dirs =
                fs::read_dir(&eu4_base_dir).expect("Failed to list EU4 saved games folders.");

            println!("Save games directory: {:?}", eu4_base_dir);
            println!();
            println!("Available save games folders:");

            for dir_entry in all_dirs {
                let Ok(dir_entry) = dir_entry else {
                    continue;
                };
                let Ok(file_type) = dir_entry.file_type() else {
                    continue;
                };
                let true = file_type.is_dir() else {
                    continue;
                };
                let dir_path = dir_entry.path();

                let mut dir_files = dir_path
                    .read_dir()
                    .expect(&format!("Failed to list path: {:?}", dir_path));

                let dir_contains_saves = dir_files.any(|file| {
                    let Ok(entry) = file else {
                        return false;
                    };
                    let Ok(ft) = entry.file_type() else {
                        return false;
                    };
                    ft.is_file() && entry.path().extension() == Some(OsStr::new(eu4_save_extension))
                });

                let dir_name = dir_path.file_name().unwrap();
                if dir_contains_saves {
                    println!("{}", dir_name.to_str().unwrap());
                }
            }
        }
        Commands::Backup => {
            println!("Backing up saves...");

            let current_savegames_dir = eu4_base_dir.join(current_savegames_dir_name);
            if !current_savegames_dir.exists() {
                println!("No save games found, skipping backup.");
                return;
            }
            
            let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
            let backup_dest_dir = eu4_base_dir.join(format!("save games {timestamp}"));
            fs::create_dir(backup_dest_dir.as_path()).expect("Cannot create backup directory");
            let copy_options = fs_extra::dir::CopyOptions::new().overwrite(true);
            if let Err(e) = fs_extra::dir::copy(&current_savegames_dir, &backup_dest_dir, &copy_options) {
                eprintln!("Backup failed: {e}");
            } else {
                let backup_dest_dir_name = backup_dest_dir.to_str().unwrap();
                println!("Backup created: {backup_dest_dir_name}");
            }
        }
    }
}
