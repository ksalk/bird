mod error;
mod saves;

use clap::{ArgGroup, Parser, Subcommand};
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
    /// List all save games folders
    List,
    /// Backup current save games folder
    Backup {
        name: Option<String>
    },
    /// Get info about current save game
    Current,
    /// Restore save games from a backed up folder
    #[command(group = ArgGroup::new("restore_source").required(true))]
    Restore {
        #[arg(short, long, group = "restore_source")]
        name: Option<String>,
        #[arg(short, long, group = "restore_source")]
        index: Option<usize>,
        /// Backup the current save games folder before restoring
        #[arg(short, long)]
        backup: bool, // TODO: make this a --no-backup flag instead??
    }
}

fn main() -> Result<(), BirdError> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::List => {
            let save_games = saves::list_save_folders()?;

            println!("{:<2} {:<8} {:<32} {:<50}", "", "Index", "Backup name", "Full backup path");
            println!("{}", "-".repeat(92)); // Separator line
            for (index, save_game) in save_games.iter().enumerate() {
                println!("{:<2} {:<8} {:<32} {:<50}", if save_game.is_current { "#" } else { "" }, index + 1, save_game.name, save_game.path.display());
            }

            Ok(())
        }
        Commands::Backup { name} => {
            match saves::backup_saves(name)? {
                Some(backup) => println!("Backup created: {}", backup.display()),
                None => println!("No save games found to backup")
            }

            Ok(())
        },
        Commands::Current => {
            let current_save_games = saves::get_current_save_games()?;

            match current_save_games {
                Some(save) => saves::read_save_data(save),
                None => Ok(())
            }
        },
        Commands::Restore { name, index, backup } => {
            println!("Restore command called with name: {:?}, index: {:?}, backup: {:?}", name, index, backup);

            let save_folder = match (name, index) {
                (Some(name), _) => saves::get_save_games_by_name(name)?,
                (_, Some(index)) => saves::get_save_games_by_index(index)?,
                _ => unreachable!("restore requires either --name or --index")
            };
            saves::restore_save(save_folder, backup)?;
            println!("Restore succeeded");

            Ok(())
        }
    }
}
