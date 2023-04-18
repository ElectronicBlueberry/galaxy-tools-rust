# Galaxy Tools Rust

Rust implementations of galaxy utility tools.
This repository only contains the cli tools themselves, not the tool wrappers.

The tool sub-folders each contain a conda recipe.
Navigate into one, and use `conda build .` to build it into a conda package.
Alternatively use `cargo build -r` to build all tools into the `target/release` folder.
Only the binaries are required, the rest are build artifacts which do not
need to be copied in order for the tool to function.

Work in progress.

## Tools

The tools in this repository can be run using `cargo run -r -p <TOOL_NAME> -- <TOOL_PARAMETERS>`

### Filter

Filter a tab-separated-value file using a simple expression.
All rows passing the expression will be kept.

Use `cargo run -r -p filter -- --help` for more information.

### Remove Beginning

Remove the first `-n` lines from a file.
Write result to a new file.

Use `cargo run -r -p remove_beginning -- --help` for more information.

## Test coverage

This repository aims to have 100% test coverage. All code should be either covered by
integration tests, contained in `tests.rs`, or as unit tests (see `filter/src/functions.rs`).

Untested code contained in `main()` should be kept to a minimum,
and only be used to parse input and relay feedback to the command line.

When adding a new tool/feature, make sure to include relevant tests.
