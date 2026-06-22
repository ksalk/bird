mod error;
mod saves;

use clap::{Parser, Subcommand};
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

fn main() -> Result<(), BirdError> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::List => {
            let save_games = saves::list_save_folders()?;

            println!("Available save games folders:");
            for save_game in save_games {
                println!("{} - {}", save_game.name, save_game.path.display());
            }

            Ok(())
        }
        Commands::Backup => {
            match saves::backup_saves()? {
                Some(backup) => println!("Backup created: {}", backup.display()),
                None => println!("No save games found to backup")
            }

            Ok(())
        }
    }
}
