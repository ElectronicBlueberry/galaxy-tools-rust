# Galaxy Tool Rust

Rust implementations of galaxy utility tools.
This repository only contains the cli tools themselves, not the tool wrappers.

Work in progress.

## Tools

The tools in this repository can be run using `cargo run -r -p <TOOL_NAME> -- <TOOL_PARAMETERS>`

### Filter

Filter a tab-separated-value file using a simple expression.
All rows passing the expression will be kept.

Use `cargo run -r -p filter -- --help` for more information.
