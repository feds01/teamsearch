# Teamsearch development commands. Run `just` to list them.

set positional-arguments

# List available recipes
default:
    @just --list --unsorted

# Install the pinned toolchain and the pre-commit hooks
setup:
    rustup toolchain install
    @command -v pre-commit >/dev/null || { echo "pre-commit not found, install it with 'brew install pre-commit' or 'pipx install pre-commit'"; exit 1; }
    pre-commit install

# Install the `teamsearch` binary from this checkout into ~/.cargo/bin
install:
    cargo install --locked --path crates/teamsearch

# Build an optimised `teamsearch` binary into target/release
build-release:
    cargo build --release -p teamsearch

# Run the CLI from source in the current directory, e.g. `just run orphans`
[no-cd]
run *args:
    cargo run -q -p teamsearch -- "$@"

# Run all tests, extra args go to `cargo test`, e.g. `just test -p teamsearch_matcher`
test *args:
    cargo test "$@"

# Run the benchmarks, extra args go to divan, e.g. `just bench matcher`
bench *args:
    cargo bench -p teamsearch_benchmark -- "$@"

# Run clippy, denying warnings
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Run clippy and the formatting check
lint: clippy fmt-check

# Run the tests, clippy and the formatting check (CI also runs `cargo deny`)
ci: test lint

# Write CHANGELOG.md with a section for the release `version`, e.g. `just changelog 0.5.0`: the pull requests merged since the last tag, by their titles
changelog version:
    @command -v git-cliff >/dev/null || { echo "git-cliff not found, install it with 'brew install git-cliff' or 'cargo install git-cliff'"; exit 1; }
    git cliff --tag "v{{version}}" --output CHANGELOG.md

# Tag and push a release from main, using the version in crates/teamsearch/Cargo.toml
release:
    #!/usr/bin/env bash
    set -euo pipefail

    fail() {
        echo "error: $1" >&2
        exit 1
    }

    # Releases are cut from an up-to-date, clean main.
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [ "$branch" != "main" ]; then
        fail "releases are cut from main (on '$branch')"
    fi
    if [ -n "$(git status --porcelain)" ]; then
        fail "the working tree is not clean"
    fi
    git fetch origin main --tags --quiet
    if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
        fail "main is not in sync with origin/main"
    fi

    # The version comes from the binary's crate, and must not be tagged yet.
    version="$(sed -n 's/^version = "\(.*\)"/\1/p' crates/teamsearch/Cargo.toml | head -1)"
    tag="v$version"
    if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
        fail "tag $tag already exists, bump the version first"
    fi

    # dist takes the release's notes from its section of the changelog.
    if ! grep -q "^## \[$version\]" CHANGELOG.md; then
        fail "CHANGELOG.md has no section for $version: run 'just changelog $version' and merge it"
    fi

    read -r -p "Release $tag? [y/N] " answer
    case "$answer" in
        y | Y) ;;
        *)
            echo "aborted"
            exit 1
            ;;
    esac

    git tag -a "$tag" -m "Release $tag"
    git push origin "$tag"

# Commit the Homebrew formula of each release in $PLAN, dist's plan, to the tap checked out at `tap`, and push it (run by the release workflow)
[no-cd]
publish-formula tap:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{tap}}"
    git config user.name "github-actions[bot]"
    git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
    export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
    brew update

    # Each release with a formula, as a line of JSON. Its formula may be named
    # other than the app.
    releases="$(jq --compact-output '.releases[] | select(any(.artifacts[]; endswith(".rb")))' <<<"$PLAN")"
    while IFS= read -r release; do
        [ -n "$release" ] || continue
        formula="$(jq --raw-output '.artifacts[] | select(endswith(".rb"))' <<<"$release")"
        version="$(jq --raw-output '.app_version' <<<"$release")"

        # Fix the formula's style, without reformatting its description or homepage.
        brew style --except-cops FormulaAudit/Homepage,FormulaAudit/Desc,FormulaAuditStrict \
            --fix "Formula/$formula" || true

        git add "Formula/$formula"
        git commit --message "${formula%.rb} $version"
    done <<<"$releases"

    git push

