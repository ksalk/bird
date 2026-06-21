use clap::{Parser, Subcommand};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    List,
    Backup
}

fn main() {
    let cli = Cli::parse();

    // TODO: Make these configurable, this depends on the OS
    let _save_games_path = "~/.local/share/Paradox Interactive/Europa Universalis IV";
    let _current_savegames_dir = "save games";

    match cli.cmd {
        Commands::List => {
            println!("Showing list of saves...")
        },
        Commands::Backup => {
            println!("Backing up saves...") 
        }
    }
}
