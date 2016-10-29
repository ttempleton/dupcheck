// dupcheck::duplicates_within finds any duplicates out of all files within the
// given directories.
// This example takes directory paths as arguments and prints the results.

extern crate dupcheck;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() {
    match env::args().count() {
        1 => {
            println!("No target directories specified.");
            return;
        },
        _ => {}
    }

    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);

    let mut dirs = vec![];

    for arg in args {
        let path = PathBuf::from(arg);

        dirs.push(path);
    }

    let dup_result = dupcheck::duplicates_within(&dirs);

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

