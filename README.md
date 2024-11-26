
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
  find     Find the code that you're looking for based on the CODEOWNERS file
  version  Command to print the version of the `teamsearch` binary
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```


```bash
$ teamsearch find . -c .github/CODEOWNERS -t "my-team" -p "some-c[o]+de-pattern"

./some/cool/path/my-team-owns/in/submodule/_here.py
119-                "context": "some-value",
119:                "some-cooode-pattern": "some-value",
120-            }
--
./another/cool/path/my-team-owns/in/_here.py
27-                ctx["context"] = get_some_value()
27:                ctx["cooode-pattern"] = "some-value"
28-            }
--
88-                reset_context(ctx)
89:                ctx["cooode-pattern"] = get_code_pattern()
90-            }
```
