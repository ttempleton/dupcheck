// dupcheck::duplicate_files takes a &[PathBuf] of file paths and checks if any
// of those files are duplicates of each other.  This is mainly intended to be
// used by the duplicates_of and duplicates_within functions but there's no
// reason it can't be used directly, if needed.
// This example takes file paths as arguments and prints the results.

extern crate dupcheck;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() {
    match env::args().count() {
        1 => {
            println!("No files specified.");
            return;
        },
        _ => {}
    }

    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);

    let mut files = vec![];

    for arg in args {
        let path = PathBuf::from(arg);

        files.push(path);
    }

    let dup_result = dupcheck::duplicate_files(&files);

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

