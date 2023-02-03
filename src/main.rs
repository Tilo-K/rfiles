extern crate chrono;
use chrono::offset::Local;
use chrono::DateTime;
use clap::{Parser, Subcommand};
use std::fs;

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

        #[arg(short, long)]
        recursive: Option<bool>,
    },
}

fn get_dir(dir: &String, recursive: bool, depth: u16) -> Vec<String> {
    let md = fs::metadata(dir).unwrap();

    if !md.is_dir() || depth > 100 {
        return vec![];
    }

    let mut result = vec![];
    let paths = fs::read_dir(dir).unwrap();

    for path in paths {
        let path = path.unwrap().path().display().to_string();
        let meta = fs::metadata(path.clone()).unwrap();

        let datetime: DateTime<Local> = meta.accessed().unwrap().into();
        let entry = format!("{}\t{}", datetime.format("%d.%m.%Y %k:%M"), &path);
        result.push(entry);

        if meta.is_dir() && recursive {
            let mut sub_dir = get_dir(&path, recursive, depth + 1);
            result.append(&mut sub_dir);
        }
    }

    return result;
}

fn list_dir(dir: String, recursive: Option<bool>) {
    let list: Vec<String>;

    match recursive {
        Some(r) => list = get_dir(&dir, r, 0),
        _ => list = get_dir(&dir, false, 0),
    }

    println!("Listing {}\nLast accesed\tFilepath\n", &dir);
    for file in list {
        println!("{}", file)
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { dir, recursive } => list_dir(dir, recursive),
    }
}