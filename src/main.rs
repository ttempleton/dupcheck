extern crate dupcheck;
#[macro_use]
extern crate clap;

use std::path::PathBuf;
use clap::{App, Arg, ArgGroup, Values};

fn values_to_paths(values: Option<Values>) -> Vec<PathBuf> {
    match values {
        Some(v) => v.map(|p| PathBuf::from(p)).collect::<Vec<PathBuf>>(),
        None => Vec::new()
    }
}

fn print_duplicates(dup_list: &[dupcheck::FileHash]) {
    if dup_list.len() > 0 {
        for duplicates in dup_list {
            println!("");
            println!("Duplicates of file {}:", duplicates.get_hash());
            for file in duplicates.get_files() {
                println!("{}", file.display());
            }
        }
    } else {
        println!("No duplicate files found.");
    }
}

fn main() {
    let matches = App::new("dupcheck")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Duplicate file checker")
        .arg(Arg::with_name("files")
            .short("o")
            .long("of")
            .empty_values(false)
            .multiple(true)
            .help("Files to check.")
        )
        .arg(Arg::with_name("directories")
            .short("w")
            .long("within")
            .empty_values(false)
            .multiple(true)
            .help("Directories to check.")
        )
        .group(ArgGroup::with_name("methods")
            .args(&["files", "directories"])
            .required(true)
            .multiple(true)
        )
        .after_help("Use both --of and --within to check the given directories \
                    for duplicates of the given files.  If only --of is used, \
                    the files' parent directories will be checked.  If only \
                    --within is used, the directories will be checked for any \
                    duplicate files.")
        .get_matches();

    let files = values_to_paths(matches.values_of("files"));
    let dirs = values_to_paths(matches.values_of("directories"));

    let dup_result = match files.is_empty() {
        true => dupcheck::duplicates_within(&dirs),
        false => {
            let dirs_opt = match dirs.is_empty() {
                true => None,
                false => Some(&dirs[..])
            };
            dupcheck::duplicates_of(&files, dirs_opt)
        }
    };

    match dup_result {
        Ok(dup_list) => print_duplicates(&dup_list),
        Err(dup_error) => println!("Error: {}", dup_error)
    };
}

