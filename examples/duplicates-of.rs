// dupcheck::duplicates_of is used to find any duplicates of given files,
// either within certain directories (if given) or within the files' parents.
// This example accepts both file and directory paths as arguments, sorting them
// into their respective lists.

extern crate dupcheck;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() {
    match env::args().count() {
        1 => {
            println!("No files or directories specified.");
            return;
        },
        _ => {}
    }

    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);

    let mut files = vec![];
    let mut dirs = vec![];
    let mut dirs_opt = None;

    for arg in args {
        let path = PathBuf::from(arg);

        if path.is_dir() {
            dirs.push(path.clone());
        }

        if path.is_file() {
            files.push(path.clone());
        }
    }

    if files.len() == 0 {
        println!("No files specified.");
        return;
    }

    if dirs.len() > 0 {
        dirs_opt = Some(&dirs[..]);
    }

    let dup_result = dupcheck::duplicates_of(&files, dirs_opt);

    if let Ok(dup_list) = dup_result {
        if dup_list.len() > 0 {
            for dup in &dup_list {
                println!("Duplicates of file {}:", dup.get_hash());
                for file in dup.get_files() {
                    println!("{}", file.display());
                }
                println!("");
            }
        } else {
            println!("No duplicate files found.");
        }
    } else {
        println!("Error: {}", dup_result.unwrap_err().description());
    }
}

