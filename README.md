
![TeamSearch Logo](docs/logo.svg)


A simple search tool built on top of `rg` to search for code with the help of `CODEOWNERS` file.

This tool ingests a valid `CODEOWNERS` file and searches for team members based on the provided search query.


## Installation

TODO: still working on distribution.

However, you can install it locally (with Rust & Cargo installed) by running:

```bash
$ cargo install --path crates/teamsearch/
```

This will install the `teamsearch` binary in your `$HOME/.cargo/bin` directory.

## Usage

Example:

```bash
Usage: teamsearch <COMMAND>

Commands:
  find     The check command checks the given files or directories for linting errors
  version  Command to print the version of the `bl` binary
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


$ teamsearch find . -c .github/CODEOWNERS -t "my-team" -p "some-c[o]+de-pattern"
```
