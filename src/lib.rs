//! Duplicate file checker.
#![deny(missing_docs)]

#[macro_use]
mod macros;
mod utilities;

use std::{
    error::Error,
    io,
    path::PathBuf,
};
use crate::utilities::PathUtilities;

/// Results of a duplicate file check, containing any duplicate file groups
/// found and any errors encountered.
pub struct DupResults {
    /// Groups of paths to duplicate files.
    duplicates: Vec<DupGroup>,

    /// Errors encountered while checking for duplicate files.
    errors: Vec<io::Error>
}

impl DupResults {
    /// Creates a new, empty `DupResults`.
    pub fn new() -> DupResults {
        DupResults {
            duplicates: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Checks for any duplicates of the specified files within their parent
    /// directories, or optionally within other specified directories, and
    /// returns the results.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any `files` are not
    /// files, any paths within `dirs_opt` are not directories or if I/O errors
    /// occur while trying to read files or directories.
    ///
    /// # Examples
    ///
    /// Check a file for duplicates within its parent directory:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![PathBuf::from("foo.txt")];
    /// let dup_result = dupcheck::DupResults::new().of(&files, None);
    /// ```
    ///
    /// Check a file for duplicates within some other directory:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![PathBuf::from("foo.txt")];
    /// let dirs = vec![PathBuf::from("bar")];
    /// let dup_result = dupcheck::DupResults::new().of(&files, Some(&dirs));
    /// ```
    pub fn of(mut self, files: &[PathBuf], dirs_opt: Option<&[PathBuf]>) -> DupResults {
        let mut check_files = vec![];

        // Make sure the files are files.
        for path in files.iter().filter(|p| !p.is_file()) {
            self.errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a file", path.display())
            ));
        }

        if let Some(dirs) = dirs_opt {

            // Check all directories for all filesizes.
            // ...but first, these are all directories, right?
            for path in dirs.iter().filter(|p| !p.is_dir()) {
                self.errors.push(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a directory", path.display())
                ));
            }

            let mut sizes = vec![];

            for file in files.iter().filter(|f| f.is_file()) {
                match file.metadata() {
                    Ok(metadata) => sizes.push(metadata.len()),
                    Err(e) => self.errors.push(io::Error::new(
                        e.kind(),
                        format!("{} ({})", file.display(), e.description())
                    ))
                };
            }

            let (mut dirs_files, mut errors) = self.get_file_paths_from_dirs(dirs, Some(&sizes));

            if !dirs_files.is_empty() {
                check_files.append(&mut dirs_files);
            }

            if !errors.is_empty() {
                self.errors.append(&mut errors);
            }

            // If the directories aren't ancestors of the files being checked,
            // the files won't be in the check list, so we need to add them.
            for file in files.iter().filter(|f| f.is_file()) {
                if !check_files.contains(file) {
                    check_files.push(file.clone());
                }
            }
        } else {

            // Check only a file's parent directory for other files of its size.
            for file in files.iter().filter(|f| f.is_file()) {
                let parent = file.parent().unwrap().to_path_buf();
                let sizes = match file.metadata() {
                    Ok(metadata) => vec![metadata.len()],
                    Err(e) => {
                        self.errors.push(io::Error::new(
                            e.kind(),
                            format!("{} ({})", file.display(), e.description())
                        ));
                        continue;
                    }
                };

                match parent.files_within(Some(&sizes)) {
                    Ok(mut files) => check_files.append(&mut files),
                    Err(e) => self.errors.push(e)
                };
            }
        }

        self.files(&check_files)
    }

    /// Checks for any duplicate files within the specified directories and
    /// returns the results.
    ///
    /// This checks for any duplicates among all files within all specified
    /// directories.  If multiple directories need to be checked separately,
    /// `within()` will need to be called individually for each case.
    ///
    /// Note that calling `within()` multiple times with the same `DupResults`
    /// will merge the results of all checks, adding newly-found duplicates to
    /// pre-existing `DupGroup`s if other duplicates of that file had been found
    /// in previous checks.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any paths within `dirs`
    /// are not directories or if I/O errors occur while trying to read files
    /// or directories.
    ///
    /// # Examples
    ///
    /// Check all files within two directories for any duplicates:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let dirs = vec![
    ///     PathBuf::from("foo"),
    ///     PathBuf::from("bar"),
    /// ];
    ///
    /// let dup_result = dupcheck::DupResults::new().within(&dirs);
    /// ```
    ///
    /// Check two directories separately for any duplicate files:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let dup_result = dupcheck::DupResults::new()
    ///     .within(&[PathBuf::from("foo")])
    ///     .within(&[PathBuf::from("bar")]);
    /// ```
    pub fn within(mut self, dirs: &[PathBuf]) -> DupResults {
        // Ensure these are actually directories.
        for path in dirs.iter().filter(|p| !p.is_dir()) {
            self.errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a directory", path.display())
            ));
        }

        let (files, mut errors) = self.get_file_paths_from_dirs(dirs, None);

        if !errors.is_empty() {
            self.errors.append(&mut errors);
        }

        self.files(&files)
    }

    /// Checks for any duplicates among the specified files and returns the
    /// results.
    ///
    /// Returns `DupResults`, which contains `DupGroup`s of the `files` found to
    /// be duplicates.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any `files` are not
    /// files or if I/O errors occur while trying to read files.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![
    ///     PathBuf::from("foo.txt"),
    ///     PathBuf::from("bar.txt"),
    /// ];
    ///
    /// let dup_result = dupcheck::DupResults::new().files(&files);
    /// ```
    pub fn files(mut self, files: &[PathBuf]) -> DupResults {
        // Make sure we're dealing with files.
        for path in files.iter().filter(|p| !p.is_file()) {
            self.errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a file", path.display())
            ));
        }

        // Organise the file paths according to file sizes.  Any file with a
        // unique size within the check list can't be a duplicate, so this will
        // ensure we don't waste time on hash checks of those files later.
        let mut sizes: Vec<(u64, Vec<PathBuf>)> = Vec::new();

        for file in files.iter().filter(|p| p.is_file()) {
            let size = match file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    self.errors.push(io::Error::new(
                        e.kind(),
                        format!("{} ({})", file.display(), e.description())
                    ));
                    continue;
                }
            };

            match sizes.iter().position(|s| s.0 == size) {
                Some(i) => sizes[i].1.push(file.clone()),
                None => sizes.push((size, vec![file.clone()]))
            };
        }

        // Check hashes of files where more than one file of its size was found.
        for size in sizes.iter().filter(|s| s.1.len() > 1) {

            for file in &size.1 {

                // If this isn't the first check for these `DupResults`, ensure
                // this file is only checked if its path hasn't been added in a
                // previous check.
                if !self.contains(file) {
                    let hash = match file.sha256() {
                        Ok(h) => h,
                        Err(e) => {
                            self.errors.push(io::Error::new(
                                e.kind(),
                                format!("{} ({})", file.display(), e.description())
                            ));
                            continue;
                        }
                    };

                    match self.duplicates.iter().position(|h| h.hash == hash) {
                        Some(i) => self.duplicates[i].add_file(file.clone()),
                        None => self.duplicates.push(DupGroup {
                            hash: hash,
                            files: vec![file.clone()]
                        })
                    };
                }
            }
        }

        // Keep only the groups with more than one file.
        self.duplicates.retain(|h| h.file_count() > 1);

        self
    }

    /// Returns a reference to the duplicate file groups.
    pub fn duplicates(&self) -> &[DupGroup] {
        &self.duplicates
    }

    /// Returns a reference to the errors.
    pub fn errors(&self) -> &[io::Error] {
        &self.errors
    }

    /// Returns the total number of all paths within all duplicate groups.
    pub fn file_count(&self) -> usize {
        let mut total = 0;

        for group in &self.duplicates {
            total += group.file_count();
        }

        total
    }

    /// Returns the paths of all files in the given directories, optionally of
    /// given sizes; and also returns any errors encountered while finding the
    /// file paths.
    fn get_file_paths_from_dirs(&self, dirs: &[PathBuf], sizes: Option<&[u64]>)
        -> (Vec<PathBuf>, Vec<io::Error>)
    {
        let mut files = Vec::new();
        let mut errors = Vec::new();

        for dir in dirs.iter().filter(|p| p.is_dir()) {
            match dir.files_within(sizes) {
                Ok(mut dir_files) => files.append(&mut dir_files),
                Err(e) => errors.push(e),
            };
        }

        (files, errors)
    }

    /// Returns whether any `DupGroup`s contain the given file path.
    fn contains(&self, path: &PathBuf) -> bool {
        let mut contains = false;

        for group in &self.duplicates {
            if group.contains(path) {
                contains = true;
                break;
            }
        }

        contains
    }
}

/// A group of duplicate files.
#[derive(Debug)]
pub struct DupGroup {
    /// The SHA-256 hash of the files in this group.
    hash: String,

    /// The paths to the duplicate files.
    files: Vec<PathBuf>
}

impl DupGroup {
    /// Returns the SHA-256 hash of the files in this group.
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    /// Returns a reference to the group's file paths.
    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Adds a file path.
    fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    /// Returns the number of file paths in this group.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    fn contains(&self, path: &PathBuf) -> bool {
        self.files.contains(path)
    }
}

