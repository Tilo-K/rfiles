use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about= None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long)]
        dir: String,
    },
}

fn list_dir(dir: String) {
    println!("{}", dir);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { dir } => list_dir(dir),
    }
}
