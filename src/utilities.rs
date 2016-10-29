use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use sha2::digest::Digest;
use sha2::sha2::Sha256;

pub trait PathUtilities {
    /// Returns a file's SHA-256 hash.
    fn sha256(&self) -> io::Result<String>;

    /// Returns all files within a directory, optionally of certain `sizes`.
    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>>;
}

impl PathUtilities for PathBuf {
    fn sha256(&self) -> io::Result<String> {
        let file = try!(File::open(&self));
        let mut hasher = Sha256::new();

        for byte in file.bytes() {
            let byte = try!(byte);
            hasher.input(&[byte]);
        }

        Ok(hasher.result_str())
    }

    fn files_within(&self, sizes: Option<&[u64]>) -> io::Result<Vec<PathBuf>> {
        let mut dirs: Vec<PathBuf> = vec![self.clone()];
        let mut files: Vec<PathBuf> = vec![];

        while !dirs.is_empty() {
            let read = try!(dirs[0].read_dir());
            let mut subdirs = vec![];

            for entry in read {
                let entry_path = try!(entry).path();

                if entry_path.is_file() {
                    let size = try!(entry_path.metadata()).len();

                    if sizes.is_none() || sizes.unwrap().contains(&size) {
                        files.push(entry_path.clone());
                    }
                }

                if entry_path.is_dir() {
                    subdirs.push(entry_path.clone());
                }
            }

            dirs.remove(0);
            dirs.append(&mut subdirs);
        }

        Ok(files)
    }
}

