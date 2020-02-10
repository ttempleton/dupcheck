use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

pub trait PathUtilities {
    /// Returns a file's SHA-256 hash.
    fn sha256(&self) -> io::Result<String>;

    /// Returns all files within a directory, optionally of certain `sizes`.
    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>>;
}

impl PathUtilities for PathBuf {
    fn sha256(&self) -> io::Result<String> {
        let bytes = fs::read(self.as_path())?;
        let mut hasher = Sha256::new();
        hasher.input(&bytes);

        Ok(format!("{:x}", hasher.result()))
    }

    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>> {
        let read_dir = try_with_path!(self.read_dir(), self);
        let mut files = Vec::new();
        let sizes_vec = match sizes {
            Some(sizes_slice) => Vec::from(sizes_slice),
            None => Vec::new(),
        };

        for entry in read_dir {
            let entry = try_with_path!(entry, self);
            let entry_path = entry.path();

            if entry_path.is_file() {
                let metadata = try_with_path!(entry_path.metadata(), entry_path);
                let size = metadata.len();

                if sizes.is_none() || sizes_vec.contains(&size) {
                    files.push(entry_path);
                }
            } else if entry_path.is_dir() {
                let mut subdir_files = entry_path.files_within(sizes)?;
                files.append(&mut subdir_files);
            }
        }

        Ok(files)
    }
}
