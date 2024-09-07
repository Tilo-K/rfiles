extern crate chrono;
use chrono::offset::Local;
use chrono::DateTime;
use clap::{Parser, Subcommand};
use glob::{glob, GlobError};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(author, version, about, long_about= None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        dir: String,

        #[clap(short, long)]
        recursive: bool,
    },
    Copy {
        source: String,
        target: String,
    },
    Delete {
        target: String,
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

fn list_dir(dir: String, recursive: bool) {
    let list: Vec<String> = get_dir(&dir, recursive, 0);

    let mut output = format!("Listing {}\nLast accesed\t\tFilepath\n\n", &dir);
    for file in list {
        output = format!("{}{}\n", output, file);
    }

    println!("{}", output);
}

fn copy_folder(src: &PathBuf, dst: &PathBuf) -> Result<usize, io::Error> {
    let mut written: usize = 0;
    let folder_name = src.file_name().expect("Invalid source folder");
    let dst_folder = dst.join(folder_name);

    fs::create_dir_all(&dst_folder).expect("Couldn't create targer folder");

    let files = fs::read_dir(src).expect("No access to source folder");

    for entry in files {
        if entry.is_err() {
            continue;
        }

        let file = entry.unwrap();
        let meta = match file.metadata() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if meta.is_dir() {
            //TODO: Probably should take out the recursion using a queue ors smth
            written += copy_folder(&file.path(), &dst_folder)?;
        } else if meta.is_file() {
            written += copy_file(&file.path(), &dst_folder.join(file.file_name()))?;
        }
    }

    Ok(written)
}

fn copy_file(src: &PathBuf, dst: &PathBuf) -> Result<usize, io::Error> {
    let src_file = File::open(src)?;
    let mut dst_file = File::create(dst)?;
    let mut written: usize = 0;
    let mut src_bytes = src_file.bytes();

    let mut b = src_bytes.next();

    while let Some(buff) = b {
        written += dst_file
            .write(&buff.expect("Error reading").to_ne_bytes())
            .expect("Error writing");
        b = src_bytes.next();
    }

    Ok(written)
}

fn copy(source: String, target: String) {
    let files = glob(&source).expect("Invalid source");
    let target_glob = glob(&target).expect("Invalid target");
    let source_paths: Vec<Result<PathBuf, GlobError>> = files.collect();
    let target_paths: Vec<Result<PathBuf, GlobError>> = target_glob.collect();

    if target_paths.len() > 1 {
        println!("Target must be a single file or directory !");
    }

    let target_path: &PathBuf = &PathBuf::from(target);
    if target_paths.len() == 1 {
        let target_path = target_paths
            .first()
            .expect("No target !")
            .as_ref()
            .expect("Invalid target");
        let target_metadata = fs::metadata(&target_path).expect("No access to target !");

        if target_metadata.is_file() && source_paths.len() > 1 {
            println!("Can't copy multiple files into a single file");
        }
    }
    if source_paths.len() == 1 {
        let src_file = source_paths
            .first()
            .expect("Invalid source file !")
            .as_ref()
            .expect("Invalid Path to source file !");

        let meta = fs::metadata(src_file).expect("No access to file !");

        if meta.is_file() {
            copy_file(&src_file, &target_path).expect("Error copying file !");
        } else if meta.is_dir() {
            copy_folder(&src_file, &target_path).expect("Error copying folder");
        }
    } else if source_paths.len() > 1 {
        //TODO: Make this more performant
        for src_file in source_paths.iter() {
            if !src_file.is_ok() {
                continue;
            }

            let src = src_file.as_ref().unwrap();
            let meta = fs::metadata(src).expect("No access to file");

            if meta.is_file() {
                let pot_name = src.file_name();
                if pot_name.is_none() {
                    continue;
                }
                let name = pot_name.unwrap();
                let target = target_path.join(name);

                let res = copy_file(&src, &target);
                if res.is_err() {
                    println!("Error copying {}", src.display());
                }
            } else if meta.is_dir() {
                let res = copy_folder(src, target_path);
                if res.is_err() {
                    eprintln!("Error copying {}", src.display());
                }
            }
        }
    }
}

fn delete(target: String) {
    let target_files = glob(&target).expect("Invalid files !");

    for file in target_files {
        if file.is_err() {
            continue;
        }

        let f = file.unwrap();
        if f.is_dir() {
            let res = fs::remove_dir_all(f.as_path());
            if res.is_err() {
                println!("Couldn't delete the folder {}", f.display());
            }
        } else {
            let res = fs::remove_file(f.as_path());
            if res.is_err() {
                println!("Couldn't delete the file {}", f.display());
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { dir, recursive } => list_dir(dir, recursive),
        Commands::Copy { source, target } => copy(source, target),
        Commands::Delete { target } => delete(target),
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
