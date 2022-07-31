use crate::duperror::DupError;
use std::fs;
use std::io;
use std::path::PathBuf;

pub(crate) trait PathUtilities {
    /// Returns a file's BLAKE3 hash.
    fn blake3(&self) -> io::Result<String>;

    /// Returns all files within a directory, optionally of certain `sizes`.
    fn files_within(&self, sizes: Option<&[u64]>) -> (Vec<PathBuf>, Vec<DupError>);
}

impl PathUtilities for PathBuf {
    fn blake3(&self) -> io::Result<String> {
        let bytes = fs::read(self.as_path())?;
        Ok(format!("{}", blake3::hash(&bytes)))
    }

    fn files_within(&self, sizes: Option<&[u64]>) -> (Vec<PathBuf>, Vec<DupError>) {
        let read_dir = match self.read_dir() {
            Ok(entries) => entries,
            Err(e) => return (vec![], vec![DupError::new(self.to_path_buf(), e)]),
        };

        let mut files = vec![];
        let mut errors = vec![];
        let sizes_vec = match sizes {
            Some(sizes_slice) => Vec::from(sizes_slice),
            None => vec![],
        };

        for entry in read_dir {
            let entry_path = match entry {
                Ok(ent) => ent.path(),
                Err(e) => {
                    errors.push(DupError::new(self.to_path_buf(), e));
                    continue;
                }
            };

            if entry_path.is_file() {
                let metadata = match entry_path.metadata() {
                    Ok(md) => md,
                    Err(e) => {
                        errors.push(DupError::new(entry_path, e));
                        continue;
                    }
                };

                let size = metadata.len();

                if sizes.is_none() || sizes_vec.contains(&size) {
                    files.push(entry_path);
                }
            } else if entry_path.is_dir() {
                let (mut sub_files, mut sub_errors) = entry_path.files_within(sizes);

                if !sub_files.is_empty() {
                    files.append(&mut sub_files);
                }

                if !sub_errors.is_empty() {
                    errors.append(&mut sub_errors);
                }
            }
        }

        (files, errors)
    }
}
