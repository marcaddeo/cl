# `cl`
`cl` is a command line tool that assists with the management of unreleased
changes when using the Keep a Changelog format in a team setting.

`cl` utilizes a `.cl` directory in the root of your git repository to store
changes. Each branch will log its changes to it's own `.yml` file that matches
the branch name. This will allow developers to log their changes in an atomic
way and avoid unnecessary merge conflicts.

## Installation

### Homebrew
```
$ brew install marcaddeo/clsuite/cl
```

## Debian
```
$ curl -LO https://github.com/marcaddeo/cl/releases/download/0.4.0/cl_0.4.0_amd64.deb
$ sudo dpkg -i cl_0.4.0_amd64.deb
```

### Linux
```
$ curl -LO https://github.com/marcaddeo/cl/releases/download/0.4.0/cl-0.4.0-x86_64-unknown-linux-musl.tar.gz
$ tar czvf cl-0.4.0-x86_64-unknown-linux-musl.tar.gz
$ sudo mv cl /usr/local/bin/cl
```

### Cargo

```
$ cargo install --git https://github.com/marcaddeo/cl
```

## Usage
```
cl 0.4.0
Marc Addeo <hi@marc.cx>
A command line tool for recording changes to be collected for use in a Keep A Changelog formatted CHANGELOG.md

USAGE:
    cl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -n, --no-headings    Hides the headings when output format is Markdown or YAML
    -h, --help           Prints help information
    -V, --version        Prints version information

OPTIONS:
    -f, --format <format>    Sets the output format to be used when displaying all changes [default: markdown]
                             [possible values: json, yaml, yml, markdown, md]

SUBCOMMANDS:
    added         Creates a change entry to be placed in the Added section of the CHANGELOG [aliases: add]
    changed       Creates a change entry to be placed in the Changed section of the CHANGELOG [aliases: change]
    deprecated    Creates a change entry to be placed in the Deprecated section of the CHANGELOG [aliases:
                  deprecate]
    removed       Creates a change entry to be placed in the Removed section of the CHANGELOG [aliases: remove]
    fixed         Creates a change entry to be placed in the Fixed section of the CHANGELOG [aliases: fix]
    security      Creates a change entry to be placed in the Security section of the CHANGELOG
    edit          Opens the change file for direct editing
```

## Examples

### Viewing unreleased changes
When running `cl` without a subcommand, all unreleased changes will be
displayed. This can be formatted as Markdown, JSON, or YAML.

```markdown
$ cl
## [Unreleased]
### Added
- Added a new feature to the website

### Removed
- Removed extraneous files from the repository

### Security
- Fixed a security flaw in the main authentication service
```

And as YAML:
```yaml
$ cl -f yaml
---
- added: Added a new feature to the website
- security: Fixed a security flaw in the main authentication service
- removed: Removed extraneous files from the repository
```

### Adding change entries
There is a subcommand for each type of change entry. Running the subcommand
will add the change entry to the appropriate change file in the storage
directory.

```
cl-added
Creates a change entry to be placed in the Added section of the CHANGELOG

USAGE:
    cl added <DESCRIPTION>...

FLAGS:
    -h, --help    Prints help information

ARGS:
    <DESCRIPTION>...    The description of this change entry
```

For example:
```
$ cl added Added a new feature to the website
```

### Editing the change file
If you make a typo or need to remove a change entry, you can run the `edit`
subcommand to open the change file in your text editor. This will try to find
your text editor using the `$VISUAL` and `$EDITOR` environment variables.

```
$ cl edit
```
