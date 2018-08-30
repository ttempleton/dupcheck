//! Duplicate file checker.
#![deny(missing_docs)]

extern crate sha2;

#[macro_use]
mod macros;
mod utilities;

use std::error::Error;
use std::io;
use std::path::PathBuf;
use utilities::PathUtilities;

/// Results of a duplicate file check.
///
/// Returns any duplicate file groups found and any errors encountered.
pub struct DupResults {
    duplicates: Vec<DupGroup>,
    errors: Vec<io::Error>
}

impl DupResults {

    /// Checks for duplicates of specified files, optionally within specified
    /// directories.
    ///
    /// If directories are specified, they will be checked; otherwise, a file's
    /// parent directory will be checked.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any `files` are not
    /// files, any paths within `dirs_opt` are not directories or if there are
    /// I/O errors while trying to read files or directories.
    ///
    /// # Examples
    ///
    /// Check a file for duplicates within its parent directory:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![PathBuf::from("foo.txt")];
    /// let dup_result = dupcheck::DupResults::of(&files, None);
    /// ```
    ///
    /// Check a file for duplicates within some other directory:
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let files = vec![PathBuf::from("foo.txt")];
    /// let dirs = vec![PathBuf::from("bar")];
    /// let dup_result = dupcheck::DupResults::of(&files, Some(&dirs));
    /// ```
    pub fn of(files: &[PathBuf], dirs_opt: Option<&[PathBuf]>) -> DupResults {
        let mut dup_errors = vec![];
        let mut check_files = vec![];

        // Make sure the files are files.
        for path in files.iter().filter(|p| !p.is_file()) {
            dup_errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a file", path.display())
            ));
        }

        if let Some(dirs) = dirs_opt {

            // Check all directories for all filesizes.
            // ...but first, these are all directories, right?
            for path in dirs.iter().filter(|p| !p.is_dir()) {
                dup_errors.push(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{} is not a directory", path.display())
                ));
            }

            let mut sizes = vec![];

            for file in files.iter().filter(|f| f.is_file()) {
                match file.metadata() {
                    Ok(metadata) => sizes.push(metadata.len()),
                    Err(e) => dup_errors.push(io::Error::new(
                        e.kind(),
                        format!("{} ({})", file.display(), e.description())
                    ))
                };
            }

            for dir in dirs.iter().filter(|d| d.is_dir()) {
                match dir.files_within(Some(&sizes)) {
                    Ok(mut files) => check_files.append(&mut files),
                    Err(e) => dup_errors.push(e)
                }
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
                        dup_errors.push(io::Error::new(
                            e.kind(),
                            format!("{} ({})", file.display(), e.description())
                        ));
                        continue;
                    }
                };

                match parent.files_within(Some(&sizes)) {
                    Ok(mut files) => check_files.append(&mut files),
                    Err(e) => dup_errors.push(e)
                };
            }
        }

        let mut dup_results = Self::files(&check_files);
        dup_results.add_errors(&mut dup_errors);

        dup_results
    }

    /// Checks for any duplicate files within the specified directories.
    ///
    /// This checks for any duplicates amongst all files within all specified
    /// directories.  If multiple directories need to be checked separately,
    /// this function will need to be called for each directory individually.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any paths within `dirs`
    /// are not directories or if there are I/O errors while trying to read
    /// files or directories.
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
    /// let dup_result = dupcheck::DupResults::within(&dirs);
    /// ```
    pub fn within(dirs: &[PathBuf]) -> DupResults {
        let mut dup_errors = vec![];

        for path in dirs.iter().filter(|p| !p.is_dir()) {
            dup_errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a directory", path.display())
            ));
        }

        let mut check_files = vec![];

        for dir in dirs.iter().filter(|p| p.is_dir()) {
            match dir.files_within(None) {
                Ok(mut files) => check_files.append(&mut files),
                Err(e) => dup_errors.push(e)
            };
        }

        let mut dup_results = Self::files(&check_files);
        dup_results.add_errors(&mut dup_errors);

        dup_results
    }

    /// Checks `files` for any duplicate files.
    ///
    /// Returns `DupResults`, which contains `DupGroup`s of the `files` found to
    /// be duplicates.
    ///
    /// # Errors
    ///
    /// The returned `DupResults` will contain errors if any `files` are not
    /// files or if there are I/O errors while trying to read files.
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
    /// let dup_result = dupcheck::DupResults::files(&files);
    /// ```
    pub fn files(files: &[PathBuf]) -> DupResults {
        let mut dup_errors = vec![];

        // Make sure we're dealing with files.
        for path in files.iter().filter(|p| !p.is_file()) {
            dup_errors.push(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a file", path.display())
            ));
        }

        // Organise the files according to their size.  Any files with a unique
        // size within the check list can't be duplicates, so this will make
        // sure we don't waste time on hash checks of those files later.
        let mut sizes: Vec<(u64, Vec<PathBuf>)> = vec![];

        for file in files.iter().filter(|p| p.is_file()) {
            let size = match file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    dup_errors.push(io::Error::new(
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
        let mut dup_groups: Vec<DupGroup> = vec![];

        for size in sizes.iter().filter(|s| s.1.len() > 1) {

            for file in &size.1 {
                let hash = match file.sha256() {
                    Ok(h) => h,
                    Err(e) => {
                        dup_errors.push(io::Error::new(
                            e.kind(),
                            format!("{} ({})", file.display(), e.description())
                        ));
                        continue;
                    }
                };

                match dup_groups.iter().position(|h| h.hash == hash) {
                    Some(i) => dup_groups[i].add_file(file.clone()),
                    None => dup_groups.push(DupGroup {
                        hash: hash,
                        files: vec![file.clone()]
                    })
                };
            }
        }

        // Keep only the hashes with more than one file associated.
        dup_groups.retain(|h| h.total_files() > 1);

        DupResults {
            duplicates: dup_groups,
            errors: dup_errors
        }
    }

    /// Returns a reference to the duplicate file groups.
    pub fn duplicates(&self) -> &Vec<DupGroup> {
        &self.duplicates
    }

    /// Returns a reference to the errors.
    pub fn errors(&self) -> &Vec<io::Error> {
        &self.errors
    }

    fn add_errors(&mut self, mut errors: &mut Vec<io::Error>) {
        self.errors.append(&mut errors);
    }

    /// Returns the total number of all paths within all duplicate groups.
    pub fn total_files(&self) -> usize {
        let mut total = 0;

        for group in &self.duplicates {
            total += group.total_files();
        }

        total
    }
}

/// A group of duplicate files.
#[derive(Debug)]
pub struct DupGroup {
    /// The SHA-256 hash of the files in this group.
    hash: String,
    /// The duplicate files.
    files: Vec<PathBuf>
}

impl DupGroup {
    /// Returns the SHA-256 hash.
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    /// Returns a reference to the group's files.
    pub fn get_files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    /// Adds a file path.
    fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    /// Returns the number of files in this group.
    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

