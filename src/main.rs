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
        let meta = fs::metadata(path.clone());
        let entry: String;
        let mut is_dir = false;

        match meta {
            Ok(data) => {
                let datetime: DateTime<Local> = data.accessed().unwrap().into();
                entry = format!("{}\t{}", datetime.format("%d.%m.%Y %k:%M"), &path);
                is_dir = data.is_dir()
            }
            _ => {
                entry = format!("No access\t\t{}", &path);
            }
        }
        result.push(entry);

        if is_dir && recursive {
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

    let mut output = format!("Listing {}\nLast accesed\t\tFilepath\n\n", &dir);
    for file in list {
        output = format!("{}{}\n", output, file);
    }

    println!("{}", output);
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { dir, recursive } => list_dir(dir, recursive),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::get_dir;

    #[test]
    fn test_get_dir_not_recursive() -> Result<(), String> {
        fs::create_dir("./test").unwrap();
        fs::File::create("./test/testA").unwrap();
        fs::File::create("./test/testB").unwrap();
        fs::File::create("./test/testC").unwrap();

        let result = get_dir(&String::from("./test"), false, 0);

        fs::remove_dir_all("./test").unwrap();

        if result.len() != 3 {
            return Err(format!(
                "Result should be of length 3 but isn't\nResult:{:?}",
                result
            ));
        }

        Ok(())
    }

    #[test]
    fn test_get_dir_recursive() -> Result<(), String> {
        fs::create_dir("./rectest").unwrap();
        fs::create_dir("./rectest/test2").unwrap();

        fs::File::create("./rectest/testA").unwrap();
        fs::File::create("./rectest/testB").unwrap();
        fs::File::create("./rectest/testC").unwrap();

        fs::File::create("./rectest/test2/testD").unwrap();

        let result = get_dir(&String::from("./rectest"), true, 0);

        fs::remove_dir_all("./rectest").unwrap();

        if result.len() != 5 {
            return Err(format!(
                "Result should be of length 5 but is {} \nResult:{:?}",
                result.len(),
                result
            ));
        }

        Ok(())
    }
}
