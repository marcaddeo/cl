# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2020-07-28
### Added
- Add an aggregate subcommand to aggregate change entries into the CHANGELOG.md
  file

### Fixed
- Fix bug that caused cl entries to be not properly nested in their branch
  directories

## [0.8.0] - 2020-07-10
### Added
- Add a subcommand to yank a specific release

### Fixed
- Ensure all cl output formats end with a newline

## [0.7.0] - 2020-02-07
### Changed
- Show all unreleased changes from CHANGELOG.md and the `.cl` directory when
  running `cl` without a subcommand

### Fixed
- Ensure the output of `cl` always ends with a newline

## [0.6.0] - 2019-10-11
### Fixed
- Fix bug that caused branch names with a directory separator to fail when
  trying to add a change entry

## [0.5.0] - 2019-10-11
### Added
- Add a build script to build releases for each target
- Add Linux (musl) installation instructions

### Changed
- Update add entry functionality to also stage the change entry in the git
  index for committing

### Fixed
- Fix error when piping the output of `cl`

## [0.4.0] - 2019-10-11
### Added
- Add installation instructions for Debian

### Removed
- Remove the `fstrings` crate so we can target Linux musl

## [0.3.0] - 2019-10-08
### Fixed
- Use the vendored-openssl feature for `git2` so we don't have to deal with it
  as a dependency

## [0.2.0] - 2019-10-08
### Added
- Add a Homebrew installation option

### Changed
- Revise error handling

## [0.1.0] - 2019-10-07
### Added
- Initial implementation of `cl` binary

[Unreleased]: https://github.com/marcaddeo/cl/compare/0.9.0...HEAD
[0.9.0]: https://github.com/marcaddeo/cl/compare/0.8.0...0.9.0
[0.8.0]: https://github.com/marcaddeo/cl/compare/0.7.0...0.8.0
[0.7.0]: https://github.com/marcaddeo/cl/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/marcaddeo/cl/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/marcaddeo/cl/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/marcaddeo/cl/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/marcaddeo/cl/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/marcaddeo/cl/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/marcadde/cl/releases/tag/0.1.0
