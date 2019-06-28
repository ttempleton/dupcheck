use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use sha2::{
    digest::Digest,
    sha2::Sha256,
};

pub trait PathUtilities {
    /// Returns a file's SHA-256 hash.
    fn sha256(&self) -> io::Result<String>;

    /// Returns all files within a directory, optionally of certain `sizes`.
    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>>;
}

impl PathUtilities for PathBuf {
    fn sha256(&self) -> io::Result<String> {
        let file = File::open(&self)?;
        let mut hasher = Sha256::new();

        for byte in file.bytes() {
            let byte = byte?;
            hasher.input(&[byte]);
        }

        Ok(hasher.result_str())
    }

    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>> {
        let read_dir = try_with_path!(self.read_dir(), self);
        let mut files = Vec::new();
        let sizes_vec = match sizes {
            Some(sizes_slice) => Vec::from(sizes_slice),
            None => Vec::new()
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

