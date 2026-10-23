![TeamSearch Logo](docs/logo.svg)

[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://app.codspeed.io/feds01/teamsearch?utm_source=badge)

`teamsearch` searches a codebase using its `CODEOWNERS` file. You can search only the code a team owns, find out who owns a file, or list the files nobody owns. It's built on ripgrep's search libraries, so it stays fast on large repositories.

## Installation

Prebuilt binaries are available for macOS, Linux and Windows. With [Homebrew](https://brew.sh), on macOS or Linux:

```sh
brew install feds01/tap/teamsearch
```

See [INSTALL.md](INSTALL.md) for the install scripts, direct downloads and building from source.

## Usage

| Command   | What it does                                    |
| --------- | ----------------------------------------------- |
| `find`    | Search for a pattern in the files a team owns   |
| `lookup`  | Show which teams own a file or directory        |
| `orphans` | List the files that no team owns                |
| `version` | Print the installed version                     |

`find`, `lookup` and `orphans` need the path to a `CODEOWNERS` file (`-c`), and check the files or directories you pass them, or the current directory if you pass none. Add `--json` for machine-readable output, and run `teamsearch help <command>` for every option.

When `find` and `orphans` walk a directory, they skip hidden files and directories (such as `.github/`), files covered by `.gitignore` inside a git repository, and common build and dependency directories such as `node_modules`, `dist` and `venv`.

The examples below use this `.github/CODEOWNERS`:

```
/web/  @acme/frontend
/api/  @acme/backend
```

### `find`

Search the code a team owns for a regular expression:

```console
$ teamsearch find -c .github/CODEOWNERS -t acme/frontend -p 'c(o)+de'
/repo/web/app.js
1:const code = 1;
2:// some cooode here
info: found 2 matches in 5.232167ms
```

- The leading `@` on a team name is optional. Repeat `-t` to search several teams, or leave it out to search every file.
- `-i` makes the pattern case-insensitive, and `--count` prints the number of matches per file.
- `-e <glob>` leaves out the paths that match a glob, such as `-e 'api/**'` or `-e '*.test.js'`.

### `lookup`

Show which teams own each file:

```console
$ teamsearch lookup -c .github/CODEOWNERS web/app.js misc/notes.txt
info: /repo/web/app.js: @acme/frontend
info: /repo/misc/notes.txt: none
```

### `orphans`

List the files that no `CODEOWNERS` rule covers, which is a quick way to spot code nobody maintains. Skipped files, such as anything under `.github/`, are never listed. Use `-e <glob>` to leave out more paths.

```console
$ teamsearch orphans -c .github/CODEOWNERS
info: /repo/scripts/deploy.sh
info: /repo/misc/notes.txt
info: found 2 files in 6.795167ms
```

## Development

### Benchmarks

The benchmarks in `crates/teamsearch_benchmark` parse a `CODEOWNERS` file, look up owners, resolve a team's files and search them, in generated repositories. [CodSpeed](https://app.codspeed.io/feds01/teamsearch) runs them on every commit to `main` and reports how each PR that touches the library crates changes them. To run them locally:

```sh
cargo bench -p teamsearch_benchmark            # every benchmark
cargo bench -p teamsearch_benchmark -- lookup  # those that match a filter
```

### Releasing

Bump `version` in `crates/teamsearch/Cargo.toml`, and with it `Cargo.lock`, and run `just changelog <version>`, which writes [CHANGELOG.md](CHANGELOG.md) with [git-cliff](https://git-cliff.org) from the titles of the pull requests merged since the last release (see [cliff.toml](cliff.toml)). Merge that to `main`, then run `just release` to tag it. [dist](https://github.com/axodotdev/cargo-dist) (`.github/workflows/release.yml`, configured in `dist-workspace.toml`) builds the binaries and installers, and publishes the GitHub release, with the release's section of the changelog as its notes, and the Homebrew formula, to [feds01/homebrew-tap](https://github.com/feds01/homebrew-tap).

A pull request's title is its entry in the changelog, so write it as a conventional commit, as in `feat(find): …`. Features, fixes, performance and documentation are listed; other types are not.

## License

MIT. See [LICENSE](LICENSE).
