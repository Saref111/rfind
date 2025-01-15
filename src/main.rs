use std::{error::Error, fs::{self, ReadDir}, thread, path::{Path, PathBuf}, str::FromStr, sync::{Arc, Mutex}};

use clap::Parser;
use num_cpus;


#[derive(Parser)]
struct Cli {
    files: Option<Vec<PathBuf>>,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    iname: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let paths = args.files.unwrap_or(vec![PathBuf::from_str(".").unwrap()]);
    let name = args.name.unwrap_or("".to_string());
    let case_insensitive = args.iname;

    let results = find_files_parallel(paths, &name, case_insensitive)?;

    for result in results {
        println!("{}", result.display());
    }
    // if let Err(e) = find_files(&paths, &name, case_insensitive) {
    //     eprintln!("Error: {e}");
    // }
    Ok(())
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


fn find_files_parallel(paths: Vec<PathBuf>, search_name: &str, is_case_insens: bool) -> Result<Vec<PathBuf>, Box<dyn Error>>{
    let (tx, rx) = std::sync::mpsc::channel();
    let search_exp = if is_case_insens { search_name.to_lowercase() } else { search_name.to_string() };

    let paths_to_process = Arc::new(Mutex::new(paths));
    let nym_threads = num_cpus::get();
    let mut handles = vec![];

    for _ in 0..nym_threads {
        let tx = tx.clone();
        let paths_to_process = Arc::clone(&paths_to_process);
        let search_exp = search_exp.clone();

        let handle = thread::spawn(move || {
            while let Some(path) = get_next_path(&paths_to_process) {
                if path.is_dir() {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            let entry_path = entry.path();

                            if entry_path.is_dir() {
                                if let Ok(mut locked) = paths_to_process.lock() {
                                    locked.push(entry_path);
                                }
                            } else if is_matching(&path, &search_exp, is_case_insens) {
                                tx.send(entry_path).unwrap();
                            }
                        }
                    }
                } else if is_matching(&path, &search_exp, is_case_insens) {
                    tx.send(path).unwrap();
                }
            };
        });

        handles.push(handle);
    }

    drop(tx);

    let mut results = vec![];

    for result in rx {
        results.push(result);
    }

    Ok(results)
}

fn get_next_path(paths_to_process: &Arc<Mutex<Vec<PathBuf>>>) -> Option<PathBuf> {
    let mut locked = paths_to_process.lock().unwrap();

    if locked.is_empty() {
        None
    } else {
        locked.pop()
    }
}