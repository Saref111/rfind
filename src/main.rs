use std::{fs, os, path::PathBuf, str::FromStr};

use clap::Parser;


#[derive(Parser)]
struct Cli {
    files: Option<Vec<PathBuf>>,

    #[arg(short, long)]
    name: Option<String>
}

fn main() {
    let args = Cli::parse();

    let paths = args.files.unwrap_or(vec![PathBuf::from_str(".").unwrap()]);
    let name = args.name.unwrap_or("".to_string());

    read(paths, name);
}

fn read(paths: Vec<PathBuf>, search_exp: String) {
    if paths.is_empty() {return;}

    let inner = paths.iter().fold(vec![], |mut acc, it| {
        if it.is_dir() {
            let inner_paths = it.read_dir().unwrap();
            let mut inner_paths: Vec<PathBuf> = inner_paths.map(|d| {
                d.unwrap().path()
            }).filter(|it| it.to_string_lossy().contains(&search_exp)).collect();

            acc.append(& mut inner_paths);
            println!("{}", it.to_string_lossy());
        }

        if it.is_file() && it.to_string_lossy().contains(&search_exp) {
            println!("{}", it.to_string_lossy());
        }

        acc
    });

    read(inner, search_exp);
}