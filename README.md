
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

```
TeamSearch: Search for large code bases with ease using CODEOWNERS

Usage: teamsearch <COMMAND>

Commands:
  find     Find the code that you're looking for based on the CODEOWNERS file
  lookup   Lookup the team that owns a specific file or directory
  version  Command to print the version of the `teamsearch` binary
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

### Searching with team domains `find`:

The `find` command is useful when you want to search for code based on a specific team and a pattern.

```py
$ teamsearch find . -c .github/CODEOWNERS -t "my-team" -p "c(o)+de"

info: match found
  --> repo/sub/item3.html:2:11
   |
 2 |     const code = {
   |           ----
   |
  ::: repo/sub/item3.html:3:15
   |
 3 |         "some-cooode-pattern": "some-value",
   |               ------
   |
  ::: repo/sub/item3.html:4:18
   |
 4 |         "another-code-pattern": "some-value",
   |                  ----
   |
  ::: repo/sub/item3.html:10:40
   |
10 |     <p>Hello world, a fast way to find code owned by teams</p>
   |                                        ----
   |
info: found 4 matches in 7.918375ms
```

### Looking up ownership with `lookup`:

A lookup is useful when you want to know which team or teams owns a specific file or directory.

```bash
$ teamsearch lookup -c .github/CODEOWNERS "some/path/my/team/owns/in/submodule/_here.py"

info: some/path/my/team/owns/in/submodule/_here.py: my-team
```
