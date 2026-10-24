# Changelog

What changed for teamsearch's users in each release. dist shows a release's
section as its notes on GitHub.

## [0.3.0] - 2026-10-24

[Changes since v0.2.0](https://github.com/feds01/teamsearch/compare/v0.2.0...v0.3.0)

### Features

- **find:** warn when a -t team isn't in CODEOWNERS ([#20](https://github.com/feds01/teamsearch/pull/20))

### Fixes

- **find:** make --no-respect-gitignore include ignored files ([#19](https://github.com/feds01/teamsearch/pull/19))

### Documentation

- tidy the README and move installation into INSTALL.md ([#18](https://github.com/feds01/teamsearch/pull/18))
- **cli:** correct the --codeowners and --json help text ([#21](https://github.com/feds01/teamsearch/pull/21))

## [0.2.0] - 2026-10-04

[Changes since v0.1.0](https://github.com/feds01/teamsearch/compare/v0.1.0...v0.2.0)

### Fixes

- **lookup:** order teams by `CODEOWNERS` declaration ([#12](https://github.com/feds01/teamsearch/pull/12))
- **cli:** pass `@team` args through when not an argfile ([#13](https://github.com/feds01/teamsearch/pull/13))

## [0.1.0] - 2026-10-02

### Fixes

- **auto-assign:** pass --repo to gh pr edit ([#7](https://github.com/feds01/teamsearch/pull/7))
- **lookup:** keep the last file when `CODEOWNERS` is an input ([#8](https://github.com/feds01/teamsearch/pull/8))

