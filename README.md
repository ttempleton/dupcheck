# dupcheck

A duplicate file checker.  dupcheck can search for duplicates of given files within their parent directories, within given directories, or for any duplicate files within given directories.

## Usage

```
dupcheck <--of <files>...|--within <directories>...>
```

* `--of` and `--within` used together will check the directories for duplicates of the files.
* `--of` used without `--within` will check the files' parent directories.
* `--within` without `--of` will check the directories for any duplicate files.

If dupcheck finds duplicate files, it will print the results in groups identified by the files' SHA-256 hashes.

## Library

The functionality of dupcheck is available for anyone who wishes to use it in their Rust project.  Add the following to the dependencies section of your Cargo.toml:

`dupcheck = "~0.1.0"`

See the [documentation](https://docs.rs/dupcheck) for more information.

## Dependencies

dupcheck uses the following crates:

* [sha2](https://crates.io/crates/sha2) ~0.8.0
* [clap](https://crates.io/crates/clap) ~2.33.0

