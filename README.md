# dupcheck

A duplicate file checker.  dupcheck can search for duplicates of given files within their parent directories, within given directories, or for any duplicate files within given directories.

## Usage

```
dupcheck <--of <files>...|--within <directories>...>
```

* `--of` and `--within` used together will check for duplicates of the given files within the given directories.
* `--of` used without `--within` will check for duplicates of the files within the files' parent directories.
* `--within` used without `--of` will check the directories for any duplicate files.

If dupcheck finds duplicate files, it will print the found files in groups identified by the files' BLAKE3 hashes.

## Library

The functionality of dupcheck is available for anyone who wishes to use it in their Rust project.  Add the following to the dependencies section of your Cargo.toml:

`dupcheck = "~0.1.0"`

See the [documentation](https://docs.rs/dupcheck) for more information.

## Dependencies

dupcheck uses the following crates:

* [blake3](https://crates.io/crates/blake3) ^1.3.1
* [clap](https://crates.io/crates/clap) ~2.33.0
