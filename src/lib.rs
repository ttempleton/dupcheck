//! Duplicate file checker.
#![deny(missing_docs)]

mod duperror;
mod utilities;

use crate::duperror::DupError;
use crate::utilities::PathUtilities;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Results of a duplicate file check, containing any duplicate file groups
/// found and any errors encountered.
pub struct DupResults {
    /// Groups of paths to duplicate files.
    duplicates: Vec<DupGroup>,

    /// Errors encountered while checking for duplicate files.
    errors: Vec<DupError>,
}

impl DupResults {
    /// Creates a new, empty `DupResults`.
    pub fn new() -> DupResults {
        DupResults {
            duplicates: vec![],
            errors: vec![],
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
    /// let mut dup_result = dupcheck::DupResults::new();
    ///
    /// if let Err(dup_error) = dup_result.of(&files, None) {
    ///     // Error handling
    /// }
    /// ```
    ///
    /// Check a file for duplicates within some other directory:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![PathBuf::from("foo.txt")];
    /// let dirs = vec![PathBuf::from("bar")];
    /// let mut dup_result = dupcheck::DupResults::new();
    ///
    /// if let Err(dup_error) = dup_result.of(&files, Some(&dirs)) {
    ///     // Error handling
    /// }
    /// ```
    pub fn of(&mut self, files: &[PathBuf], dirs_opt: Option<&[PathBuf]>) -> io::Result<()> {
        self.check_valid_paths(Some(files), dirs_opt)?;

        let mut check_files = vec![];

        if let Some(dirs) = dirs_opt {
            let mut sizes = vec![];

            for file in files {
                match file.metadata() {
                    Ok(metadata) => sizes.push(metadata.len()),
                    Err(e) => self.errors.push(DupError::new(file.to_path_buf(), e)),
                };
            }

            let (mut dirs_files, mut errors) = self.files_within(dirs, Some(&sizes));

            if !dirs_files.is_empty() {
                check_files.append(&mut dirs_files);
            }

            if !errors.is_empty() {
                self.errors.append(&mut errors);
            }

            // If the directories aren't ancestors of the files being checked,
            // the files won't be in the check list, so we need to add them.
            for file in files {
                if !check_files.contains(file) {
                    check_files.push(file.to_path_buf());
                }
            }
        } else {
            // Check only a file's parent directory for other files of its size.
            for file in files {
                let parent = file.parent().unwrap().to_path_buf();
                let sizes = match file.metadata() {
                    Ok(metadata) => vec![metadata.len()],
                    Err(e) => {
                        self.errors.push(DupError::new(file.to_path_buf(), e));
                        continue;
                    }
                };

                let (mut p_files, mut p_errors) = parent.files_within(Some(&sizes));

                if !p_files.is_empty() {
                    check_files.append(&mut p_files);
                }

                if !p_errors.is_empty() {
                    self.errors.append(&mut p_errors);
                }
            }
        }

        self._files(&check_files)
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
    /// let mut dup_result = dupcheck::DupResults::new();
    /// let dirs = vec![
    ///     PathBuf::from("foo"),
    ///     PathBuf::from("bar"),
    /// ];
    ///
    /// if let Err(dup_error) = dup_result.within(&dirs) {
    ///     // Error handling
    /// }
    /// ```
    ///
    /// Check two directories separately for any duplicate files:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let mut dup_result = dupcheck::DupResults::new();
    /// let dirs = [
    ///     [PathBuf::from("foo")],
    ///     [PathBuf::from("bar")],
    /// ];
    ///
    /// for dir in &dirs {
    ///     if let Err(dup_error) = dup_result.within(dir) {
    ///         // Error handling
    ///     }
    /// }
    /// ```
    pub fn within(&mut self, dirs: &[PathBuf]) -> io::Result<()> {
        self.check_valid_paths(None, Some(dirs))?;

        let (files, mut errors) = self.files_within(dirs, None);

        if !errors.is_empty() {
            self.errors.append(&mut errors);
        }

        self._files(&files)
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
    /// let mut dup_result = dupcheck::DupResults::new();
    /// let files = vec![
    ///     PathBuf::from("foo.txt"),
    ///     PathBuf::from("bar.txt"),
    /// ];
    ///
    /// if let Err(dup_error) = dup_result.files(&files) {
    ///     // Error handling
    /// }
    /// ```
    pub fn files(&mut self, files: &[PathBuf]) -> io::Result<()> {
        self.check_valid_paths(Some(files), None)?;

        self._files(&files)
    }

    fn _files(&mut self, files: &[PathBuf]) -> io::Result<()> {
        // Organise the file paths according to file sizes.  Any file with a
        // unique size within the check list can't be a duplicate, so this will
        // ensure we don't waste time on hash checks of those files later.
        let mut file_paths_by_size = HashMap::new();

        for file in files {
            let size = match file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    self.errors.push(DupError::new(file.to_path_buf(), e));
                    continue;
                }
            };

            if !file_paths_by_size.contains_key(&size) {
                file_paths_by_size.insert(size, vec![]);
            }

            file_paths_by_size.get_mut(&size).unwrap().push(file.clone());
        }

        // Check hashes of files where more than one file of its size was found.
        for paths in file_paths_by_size.values().filter(|v| v.len() > 1) {
            for file in paths {
                // If this isn't the first check for these `DupResults`, ensure
                // this file is only checked if its path hasn't been added in a
                // previous check.
                if !self.contains(file) {
                    let hash = match file.blake2() {
                        Ok(h) => h,
                        Err(e) => {
                            self.errors.push(DupError::new(file.to_path_buf(), e));
                            continue;
                        }
                    };

                    match self.duplicates.iter().position(|h| h.hash == hash) {
                        Some(i) => self.duplicates[i].add_file(file.clone()),
                        None => self.duplicates.push(DupGroup {
                            hash: hash,
                            files: vec![file.clone()],
                        }),
                    };
                }
            }
        }

        // Keep only the groups with more than one file.
        self.duplicates.retain(|h| h.file_count() > 1);

        Ok(())
    }

    /// Returns a reference to the duplicate file groups.
    pub fn duplicates(&self) -> &[DupGroup] {
        &self.duplicates
    }

    /// Returns a reference to the errors.
    pub fn errors(&self) -> &[DupError] {
        &self.errors
    }

    /// Returns the total number of all paths within all duplicate groups.
    pub fn file_count(&self) -> usize {
        self.duplicates.iter().fold(0, |acc, g| acc + g.file_count())
    }

    /// Returns the paths of all files in the given directories, optionally of
    /// given sizes; and also returns any errors encountered while finding the
    /// file paths.
    fn files_within(
        &self,
        dirs: &[PathBuf],
        sizes: Option<&[u64]>,
    ) -> (Vec<PathBuf>, Vec<DupError>) {
        let mut files = vec![];
        let mut errors = vec![];

        for dir in dirs {
            let (mut dir_files, mut dir_errors) = dir.files_within(sizes);

            if !dir_files.is_empty() {
                files.append(&mut dir_files);
            }

            if !dir_errors.is_empty() {
                errors.append(&mut dir_errors);
            }
        }

        (files, errors)
    }

    /// Returns whether any `DupGroup`s contain the given file path.
    fn contains(&self, path: &PathBuf) -> bool {
        self.duplicates.iter().any(|g| g.contains(path))
    }

    fn check_valid_paths(&self, files: Option<&[PathBuf]>, dirs: Option<&[PathBuf]>) -> io::Result<()> {
        if let Some(unwrapped_files) = files {
            if let Some(path) = unwrapped_files.iter().find(|p| !p.is_file()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a file", path.display()),
                ));
            }
        }

        if let Some(unwrapped_dirs) = dirs {
            if let Some(path) = unwrapped_dirs.iter().find(|p| !p.is_dir()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a directory", path.display()),
                ));
            }
        }

        Ok(())
    }
}

/// A group of duplicate files.
#[derive(Debug)]
pub struct DupGroup {
    /// The Blake2 hash of the files in this group.
    hash: String,

    /// The paths to the duplicate files.
    files: Vec<PathBuf>,
}

impl DupGroup {
    /// Returns the Blake2 hash of the files in this group.
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
