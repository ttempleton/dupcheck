/// Unwraps a successful result or returns the error, with the path included in
/// the error description.
macro_rules! try_with_path {
    ($x:expr, $y:expr) => {{
        match $x {
            Ok(r) => r,
            Err(e) => return Err(DupError::new($y.to_path_buf(), e)),
        }
    }};
}
