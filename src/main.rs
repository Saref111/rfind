use std::{error::Error, fs::{self, ReadDir}, path::{Path, PathBuf}, str::FromStr};

use clap::Parser;


#[derive(Parser)]
struct Cli {
    files: Option<Vec<PathBuf>>,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    iname: bool,
}

fn main() {
    let args = Cli::parse();

    let paths = args.files.unwrap_or(vec![PathBuf::from_str(".").unwrap()]);
    let name = args.name.unwrap_or("".to_string());
    let case_insensitive = args.iname;

    if let Err(e) = find_files(&paths, &name, case_insensitive) {
        eprintln!("Error: {e}");
    }
}

fn find_files(paths: &[PathBuf], search_name: &str, is_case_insens: bool) -> Result<(), Box<dyn Error>>{
    if paths.is_empty() {return Ok(());}
    let search_exp = if is_case_insens { search_name.to_lowercase() } else { search_name.to_string() };

    let mut dirs_to_explore = paths.to_vec();

    while let Some(path) = dirs_to_explore.pop() {
        if path.is_dir() {
            match fs::read_dir(&path) {
                Ok(entries) => {
                    handle_dir_entries(entries, &mut dirs_to_explore, &search_exp, is_case_insens)?
                },
                Err(e) => {
                    eprintln!("Cannot read dir {}: {}", path.display(), e);
                }
            }
        } else if is_matching(&path, &search_exp, is_case_insens) {
            println!("{}", path.display());
        }
    }

    Ok(())
}

fn is_matching(path: &Path, search_exp: &str, is_case_insens: bool) -> bool {
    path.file_name().and_then(|name| name.to_str()).map(|name| {
        if is_case_insens {
            name.to_lowercase().contains(search_exp)
        } else {
            name.contains(search_exp)
        }
    }).unwrap_or(false)
}

fn handle_dir_entries(entries: ReadDir, dirs_to_explore: &mut Vec<PathBuf>, search_exp: &str, is_case_insens: bool) -> Result<(), Box<dyn Error>> {
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            dirs_to_explore.push(entry_path);
        } else if is_matching(&entry_path, search_exp, is_case_insens) {
            println!("{}", entry_path.display());
        }
    }

    Ok(())
}