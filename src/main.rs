use clap::{crate_authors, crate_version, App, Arg, ArgGroup, Values};
use std::io;
use std::path::PathBuf;

fn values_to_paths(values: Option<Values>) -> Vec<PathBuf> {
    match values {
        Some(v) => v.map(|p| PathBuf::from(p)).collect::<Vec<PathBuf>>(),
        None => vec![],
    }
}

fn get_dup_result(files: &[PathBuf], dirs: &[PathBuf]) -> io::Result<dupcheck::DupResults> {
    let mut dup_result = dupcheck::DupResults::new();

    if files.is_empty() {
        dup_result.within(&dirs)?;
    } else {
        let dirs_opt = match dirs.is_empty() {
            true => None,
            false => Some(&dirs[..]),
        };
        dup_result.of(&files, dirs_opt)?;
    }

    Ok(dup_result)
}

fn print_duplicates(dup_list: &dupcheck::DupGroup) {
    println!();
    println!("Duplicates of file {}:", dup_list.get_hash());
    for file in dup_list.get_files() {
        println!("{}", file.display());
    }
}

fn main() {
    let matches = App::new("dupcheck")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Duplicate file checker")
        .arg(
            Arg::with_name("files")
                .short("o")
                .long("of")
                .empty_values(false)
                .multiple(true)
                .help("Files to check."),
        )
        .arg(
            Arg::with_name("directories")
                .short("w")
                .long("within")
                .empty_values(false)
                .multiple(true)
                .help("Directories to check."),
        )
        .group(
            ArgGroup::with_name("methods")
                .args(&["files", "directories"])
                .required(true)
                .multiple(true),
        )
        .after_help(
            "Use both --of and --within to check the given directories \
                    for duplicates of the given files.  If only --of is used, \
                    the files' parent directories will be checked.  If only \
                    --within is used, the directories will be checked for any \
                    duplicate files.",
        )
        .get_matches();

    let files = values_to_paths(matches.values_of("files"));
    let dirs = values_to_paths(matches.values_of("directories"));

    let dup_result = get_dup_result(&files, &dirs);

    if let Ok(dup_results) = dup_result {
        let file_count = dup_results.file_count();
        let group_count = dup_results.duplicates().len();
        let dup_errors = dup_results.errors();
        let dup_error_count = dup_errors.len();

        println!(
            "{} files found in {} group{}.",
            file_count,
            group_count,
            if group_count != 1 { "s" } else { "" }
        );

        for dup_group in dup_results.duplicates() {
            print_duplicates(&dup_group);
        }

        if dup_error_count > 0 {
            println!(
                "\n{} error{}.",
                dup_error_count,
                if dup_error_count != 1 { "s" } else { "" }
            );

            for dup_error in dup_errors {
                println!("{}", dup_error)
            }
        }
    } else if let Err(dup_error) = dup_result {
        println!("{}", dup_error);
    }
}
