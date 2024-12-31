use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DupError {
	path: PathBuf,
	io_error: io::Error,
}

impl DupError {
	pub fn new(path: PathBuf, io_error: io::Error) -> DupError {
		DupError { path, io_error }
	}
}

impl fmt::Display for DupError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{} ({})", self.path.display(), self.io_error)
	}
}

impl Error for DupError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.io_error)
	}
}
