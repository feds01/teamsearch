# Installing teamsearch

Every [release](https://github.com/feds01/teamsearch/releases) ships prebuilt binaries, so you don't need a Rust toolchain:

| Platform               | Archive                                        |
| ---------------------- | ---------------------------------------------- |
| macOS, Apple Silicon   | `teamsearch-aarch64-apple-darwin.tar.xz`       |
| macOS, Intel           | `teamsearch-x86_64-apple-darwin.tar.xz`        |
| Linux, x86_64 (static) | `teamsearch-x86_64-unknown-linux-musl.tar.xz`  |
| Linux, arm64 (static)  | `teamsearch-aarch64-unknown-linux-musl.tar.xz` |
| Windows, x86_64        | `teamsearch-x86_64-pc-windows-msvc.zip`        |

## Install script

The install script picks the right archive for your machine and puts `teamsearch` in `$XDG_BIN_HOME` if that is set, else in `$XDG_DATA_HOME/../bin` if that is set, else in `~/.local/bin`. It adds that directory to your `PATH`, so open a new shell afterwards. Set `TEAMSEARCH_NO_MODIFY_PATH=1` to leave your `PATH` alone.

macOS and Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/feds01/teamsearch/releases/latest/download/teamsearch-installer.sh | sh
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/feds01/teamsearch/releases/latest/download/teamsearch-installer.ps1 | iex"
```

## Download an archive

To skip the installer, for example in CI, download the archive for your platform and unpack it. The binary ends up in a folder named after the archive, e.g. `teamsearch-aarch64-apple-darwin/teamsearch`.

```sh
curl -LO https://github.com/feds01/teamsearch/releases/latest/download/teamsearch-aarch64-apple-darwin.tar.xz
tar -xf teamsearch-aarch64-apple-darwin.tar.xz
```

Each archive has a matching `.sha256` file you can check the download against.

## Build from source

With Rust 1.88 or newer installed:

```sh
cargo install --git https://github.com/feds01/teamsearch teamsearch
```

## Check the install

```sh
teamsearch version
```
