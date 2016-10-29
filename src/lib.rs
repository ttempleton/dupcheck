//! Duplicate file checking functions for Rust.
#![deny(missing_docs)]

extern crate sha2;

#[macro_use]
mod macros;
mod utilities;

use std::error::Error;
use std::io;
use std::path::PathBuf;
use utilities::PathUtilities;

/// Keeps information about a file hash and the files with that hash.
#[derive(Debug)]
pub struct FileHash {
    /// A SHA-256 hash.
    hash: String,
    /// The files with that hash.
    files: Vec<PathBuf>
}

impl FileHash {
    /// Returns the SHA-256 hash.
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    /// Returns a reference to the files associated with this hash.
    pub fn get_files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    /// Returns both the hash and the files reference.
    pub fn get_hash_and_files(&self) -> (String, &Vec<PathBuf>) {
        (self.hash.clone(), &self.files)
    }

    /// Adds a file path.
    pub fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    /// Returns the total number of files.
    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

/// Checks for duplicates of specified files.
///
/// If directories are specified, they will be checked; otherwise, a file's
/// parent directory will be checked.
///
/// # Errors
///
/// Returns an error if any `files` are not files, any paths within `dirs_opt`
/// are not directories or if there are I/O errors while trying to read files
/// or directories.
///
/// # Examples
///
/// Check a file for duplicates within its parent directory:
///
/// ```
/// use std::path::PathBuf;
///
/// let files = vec![PathBuf::from("foo.txt")];
/// let dup_result = dupcheck::duplicates_of(&files, None);
/// ```
///
/// Check a file for duplicates within some other directory:
///
/// ```
/// use std::path::PathBuf;
///
/// let files = vec![PathBuf::from("foo.txt")];
/// let dirs = vec![PathBuf::from("bar")];
/// let dup_result = dupcheck::duplicates_of(&files, Some(&dirs));
/// ```
pub fn duplicates_of(files: &[PathBuf], dirs_opt: Option<&[PathBuf]>)
    -> io::Result<Vec<FileHash>>
{
    let mut check_files = vec![];

    // Make sure the files are files.
    for path in files.iter().filter(|p| !p.is_file()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a file", path.display())
        ));
    }

    if let Some(dirs) = dirs_opt {

        // Check all directories for all filesizes.
        // ...but first, these are all directories, right?
        for path in dirs.iter().filter(|p| !p.is_dir()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a directory", path.display())
            ));
        }

        let mut sizes = vec![];

        for file in files {
            let metadata = try_with_path!(file.metadata(), file);
            sizes.push(metadata.len());
        }

        for dir in dirs {
            let mut dir_files = try_with_path!(
                dir.files_within(Some(&sizes)),
                dir
            );

            check_files.append(&mut dir_files);
        }

        // If the directories aren't ancestors of the files being checked, the
        // files won't be in the check list, so we need to add them.
        for file in files {
            if !check_files.contains(file) {
                check_files.push(file.clone());
            }
        }
    } else {

        // Check only a file's parent directory for other files of its size.
        for file in files {
            let parent = file.parent().unwrap().to_path_buf();
            let metadata = try_with_path!(file.metadata(), file);
            let sizes = vec![metadata.len()];

            let mut dir_files = try_with_path!(
                parent.files_within(Some(&sizes)),
                parent
            );

            check_files.append(&mut dir_files);
        }
    }

    duplicate_files(&check_files)
}

/// Checks for any duplicate files within the specified directories.
///
/// This checks for any duplicates amongst all files within all specified
/// directories.  If multiple directories need to be checked separately, this
/// function will need to be called for each directory individually.
///
/// # Errors
///
/// Returns an error if any paths within `dirs` are not directories or if there
/// are I/O errors while trying to read files or directories.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// let dirs = vec![
///     PathBuf::from("foo"),
///     PathBuf::from("bar")
/// ];
///
/// let dup_result = dupcheck::duplicates_within(&dirs);
/// ```
pub fn duplicates_within(dirs: &[PathBuf]) -> io::Result<Vec<FileHash>> {
    for path in dirs.iter().filter(|p| !p.is_dir()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", path.display())
        ));
    }

    let mut check_files = vec![];

    for dir in dirs {
        let mut dir_files = try_with_path!(dir.files_within(None), dir);

        check_files.append(&mut dir_files);
    }

    duplicate_files(&check_files)
}

/// Checks `files` for any duplicate files.
///
/// Returns the SHA-256 hashes, and the paths associated, of those found to be
/// duplicates.  Each hash/files group is represented by a `FileHash`.
///
/// # Errors
///
/// Returns an error if any `files` are not files or if there are I/O errors
/// while trying to read files.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// let files = vec![
///     PathBuf::from("foo.txt"),
///     PathBuf::from("bar.txt")
/// ];
///
/// let dup_result = dupcheck::duplicate_files(&files);
/// ```
pub fn duplicate_files(files: &[PathBuf]) -> io::Result<Vec<FileHash>> {

    // Make sure we're dealing with files.
    for path in files.iter().filter(|p| !p.is_file()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a file", path.display())
        ));
    }

    // Organise the files according to their size.  Any files with a unique
    // size within the check list can't be duplicates, so this will make sure
    // we don't waste time on hash checks of those files later.
    let mut sizes: Vec<(u64, Vec<PathBuf>)> = vec![];

    for file in files {
        let metadata = try_with_path!(file.metadata(), file);
        let size = metadata.len();

        if let Some(i) = sizes.iter().position(|s| s.0 == size) {
            sizes[i].1.push(file.clone())
        } else {
            sizes.push((size, vec![file.clone()]));
        }
    }

    // Check hashes of files where more than one file of its size was found.
    let mut hash_list: Vec<FileHash> = vec![];

    for size in sizes.iter().filter(|s| s.1.len() > 1) {

        for file in &size.1 {
            let hash = try_with_path!(file.sha256(), file);

            if let Some(i) = hash_list.iter().position(|h| h.hash == hash) {
                hash_list[i].add_file(file.clone());
            } else {
                hash_list.push(FileHash {
                    hash: hash,
                    files: vec![file.clone()]
                });
            }
        }
    }

    // Remove hashes with only one file associated.
    let mut remove: Vec<usize> = vec![];

    for (i, hash) in hash_list.iter().enumerate() {
        if hash.total_files() == 1 {
            remove.push(i);
        }
    }

    for r in remove.iter().rev() {
        hash_list.remove(*r);
    }

    Ok(hash_list)
}

