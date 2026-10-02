
![TeamSearch Logo](docs/logo.svg)


A simple search tool built on top of `rg` to search for code with the help of `CODEOWNERS` file.

This tool ingests a valid `CODEOWNERS` file and searches for team members based on the provided search query.


## Installation

Prebuilt binaries for macOS (Apple Silicon, Intel), Linux (x86_64, arm64; statically linked) and Windows ship with every [release](https://github.com/feds01/teamsearch/releases). No Rust toolchain needed.

macOS / Linux:

```bash
$ curl --proto '=https' --tlsv1.2 -LsSf https://github.com/feds01/teamsearch/releases/latest/download/teamsearch-installer.sh | sh
```

Windows:

```powershell
> powershell -ExecutionPolicy Bypass -c "irm https://github.com/feds01/teamsearch/releases/latest/download/teamsearch-installer.ps1 | iex"
```

The installer puts `teamsearch` in `~/.local/bin` (or `$XDG_BIN_HOME`). To fetch an archive directly instead, e.g. in CI:

```bash
$ gh release download --repo feds01/teamsearch --pattern 'teamsearch-aarch64-apple-darwin.tar.xz'
```

From source (with Rust & Cargo installed):

```bash
$ cargo install --git https://github.com/feds01/teamsearch teamsearch
```

### Releasing

Releases are built by [dist](https://github.com/axodotdev/cargo-dist) (`.github/workflows/release.yml`, config in `dist-workspace.toml`). Bump `version` in `crates/teamsearch/Cargo.toml`, merge, then push a matching tag:

```bash
$ git tag v0.1.1 && git push origin v0.1.1
```

## Usage

Example:

```
TeamSearch: Search for large code bases with ease using CODEOWNERS

Usage: teamsearch <COMMAND>

Commands:
  find     Find the code that you're looking for based on the CODEOWNERS file
  lookup   Lookup the team that owns a specific file or directory
  orphans  Command to find orphaned files in a project
  version  Command to print the version of the `teamsearch` binary
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

### Searching with team domains `find`:

The `find` command is useful when you want to search for code based on a specific team and a pattern.

```bash
teamsearch find . -c .github/CODEOWNERS -t "my-team" -p "c(o)+de"
```

```
repo/sub/item3.html
2:    const code = {
3:        "some-cooode-pattern": "some-value",
4:        "another-code-pattern": "some-value",
10:    <p>Hello world, a fast way to find code owned by teams</p>

info: found 4 matches in 7.918375ms
```

### Looking up ownership with `lookup`:

A lookup is useful when you want to know which team or teams owns a specific file or directory.

```bash
teamsearch lookup -c .github/CODEOWNERS "some/path/my/team/owns/in/submodule/_here.py"
```

```bash
info: some/path/my/team/owns/in/submodule/_here.py: my-team
```

### Identifying files that aren't owned with  `orphans`:

This command is useful for finding files within a project that are governed by
a `CODEOWNERS` file, but are not owned by anyone. This can be useful for
finding files that are not being maintained, or are not being maintained
properly.

```bash
teamsearch orphans -c .github/CODEOWNERS .
```

```bash
info: some/path/my/team/owns/in/submodule/_here.py
info: some/path/my/team/owns/in/othermodule/_here.py
info: some/path/other/team/owns/in/submodule/_here.py
info: found 3 files in 7.918375ms
```
