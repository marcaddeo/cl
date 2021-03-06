use anyhow::{bail, Result};
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, AppSettings, Arg,
    SubCommand,
};
use clparse::ChangelogParser;
use clparse::changelog::{Change, Changelog, ReleaseBuilder};
use err_derive::Error;
use semver::Version;
use scan_dir::ScanDir;
use gag::Gag;
use remove_empty_subdirs::remove_empty_subdirs;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Write};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Error)]
enum ClError {
    #[error(
        display = "there was an error when attempting to scan the .cl directory for change files"
    )]
    ScanError(#[error(source)] Vec<scan_dir::Error>),
    #[error(display = "could not determine determine the repository root. are you in a git repo?")]
    RepositoryError(#[error(source)] git2::Error),
    #[error(display = "invalid release version string: {}", _0)]
    ReleaseStringError(#[error(source)] semver::SemVerError),
    #[error(display = "could not find release {}", _0)]
    ReleaseNotFound(String),
    #[error(display = "could not build release for output: {}", _0)]
    ErrorBuildingRelease(String),
    #[error(display = "could not determine the repository HEAD")]
    CouldNotDetermineHead,
}

fn main() -> Result<()> {
    let matches = app_from_crate!()
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("format")
                .help("Sets the output format to be used when displaying all changes")
                .takes_value(true)
                .default_value("markdown")
                .possible_values(&["json", "yaml", "yml", "markdown", "md"])
                .short("f")
                .long("format"),
        )
        .arg(
            Arg::with_name("no-headings")
                .help("Hides the headings when output format is Markdown or YAML")
                .short("n")
                .long("no-headings")
        )
        .subcommand(
            SubCommand::with_name("added")
                .visible_alias("add")
                .about("Creates a change entry to be placed in the Added section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("changed")
                .visible_alias("change")
                .about("Creates a change entry to be placed in the Changed section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("deprecated")
                .visible_alias("deprecate")
                .about("Creates a change entry to be placed in the Deprecated section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("removed")
                .visible_alias("remove")
                .about("Creates a change entry to be placed in the Removed section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("fixed")
                .visible_alias("fix")
                .about("Creates a change entry to be placed in the Fixed section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("security")
                .about("Creates a change entry to be placed in the Security section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .required(true)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Opens the change file for direct editing")
        )
        .subcommand(
            SubCommand::with_name("yank")
                .about("Mark a specific release as [YANKED]")
                .arg(
                    Arg::with_name("release")
                        .help("The release to mark as [YANKED]")
                        .value_name("RELEASE")
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("aggregate")
                .about("Aggregate change entries into the Unreleased section of the CHANGELOG")
        )
        .get_matches();

    match matches.subcommand() {
        ("", None) => {
            let changes = get_all_changes()?;
            let output = match matches.value_of("format").unwrap() {
                "json" => {
                    format!("{}\n", serde_json::to_string_pretty(&changes)?)
                }
                "yaml" | "yml" => {
                    let mut output = serde_yaml::to_string(&changes)?;
                    if matches.is_present("no-headings") {
                        output = output.replace("---\n", "");
                    }
                    format!("{}\n", output.to_string().trim_end())
                }
                "markdown" | "md" => {
                    let release = ReleaseBuilder::default()
                        .changes(changes)
                        .build()
                        .map_err(ClError::ErrorBuildingRelease)?;

                    let mut output = format!("{}", release);
                    if matches.is_present("no-headings") {
                        output = output.replace("## [Unreleased]\n", "");
                    }
                    format!("{}\n", output.to_string().trim_end())
                }
                _ => unreachable!(),
            };

            io::stdout().write_all(output.as_bytes())?;
            Ok(())
        }
        ("edit", Some(_)) => {
            let cl_path = get_cl_path()?.into_os_string();
            let cl_path = cl_path.to_str().unwrap();
            let options = (env::var("VISUAL").ok(), env::var("EDITOR").ok());

            let editor = match options {
                (Some(visual), _) => visual,
                (_, Some(editor)) => editor,
                _ => panic!("Neither $VISUAL nor $EDITOR were set"),
            };

            Command::new(editor).arg(cl_path).spawn()?.wait()?;

            Ok(())
        }
        ("yank", Some(sub_matches)) => {
            let version_arg = sub_matches.value_of("release").unwrap();
            let version = Version::parse(version_arg)
                .map_err(ClError::ReleaseStringError)?;
            let mut changelog = get_changelog()?;

            if let Some(release) = changelog.release_mut(version) {
                release.yank(true);
                std::fs::write(get_changelog_path()?, format!("{}", changelog))?;
            } else {
                bail!(ClError::ReleaseNotFound(version_arg.to_string()));
            }

            Ok(())
        }
        ("aggregate", Some(_)) => {
            let mut changelog = get_changelog()?;

            if let Some(unreleased) = changelog.unreleased_mut() {
                unreleased.set_changes(get_all_changes()?);
                std::fs::write(get_changelog_path()?, format!("{}", changelog))?;
                remove_cl_change_entries()?;
            } else {
                bail!(ClError::ReleaseNotFound("Unreleased".to_string()));
            }

            Ok(())
        }
        (kind, Some(sub_matches)) => {
            let description = sub_matches
                .values_of("description")
                .unwrap()
                .collect::<Vec<_>>()
                .join(" ");

            add_change(Change::new(kind, description.to_string()).unwrap())?;

            Ok(())
        }
        _ => Ok(()),
    }
}

fn add_change(change: Change) -> Result<()> {
    let cl_path = get_cl_path()?;
    let mut changes = get_changes(cl_path.clone())?;
    changes.push(change);

    let contents = format!("{}\n", serde_yaml::to_string(&changes)?);
    std::fs::write(cl_path.clone(), contents)?;

    let repo = git2::Repository::discover(".").map_err(ClError::RepositoryError)?;
    let cl_path_relative = cl_path.strip_prefix(repo.path().parent().unwrap())?;
    let mut index = repo.index()?;
    index.add_path(&cl_path_relative)?;
    index.write()?;

    Ok(())
}

fn get_cl_dir() -> Result<PathBuf> {
    let repo = git2::Repository::discover(".").map_err(ClError::RepositoryError)?;
    let cl_path = PathBuf::from(repo.path()).with_file_name(".cl");

    Ok(cl_path)
}

fn get_cl_path() -> Result<PathBuf> {
    let repo = git2::Repository::discover(".").map_err(ClError::RepositoryError)?;
    let head = repo.head()?;
    let head = head.shorthand().ok_or(ClError::CouldNotDetermineHead)?;
    let mut cl_path = get_cl_dir()?;

    cl_path.push(head);
    create_dir_all(cl_path.clone())?;
    cl_path.push("changes.yml");

    Ok(cl_path)
}

fn get_changes(cl_path: PathBuf) -> Result<Vec<Change>> {
    let mut contents = String::new();
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(cl_path)?;
    file.read_to_string(&mut contents)?;

    if !contents.is_empty() {
        Ok(serde_yaml::from_str(&contents)?)
    } else {
        Ok(Vec::new())
    }
}

fn get_changelog_path() -> Result<PathBuf> {
    let repo = git2::Repository::discover(".").map_err(ClError::RepositoryError)?;
    Ok(PathBuf::from(repo.path()).with_file_name("CHANGELOG.md"))
}

fn get_changelog() -> Result<Changelog> {
    Ok(ChangelogParser::parse(get_changelog_path()?)?)
}

fn get_unreleased_changes() -> Result<Vec<Change>> {
    Ok(get_changelog()?.unreleased_changes())
}

fn get_cl_entry_paths() -> Result<Vec<PathBuf>> {
    let mut logs: Vec<PathBuf> = Vec::new();
    let cl_dir = get_cl_dir()?;

    ScanDir::files()
        .walk(cl_dir, |iter| {
            for (entry, _) in iter {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "yml" {
                        logs.push(path);
                    }
                }
            }
        })
        .map_err(ClError::ScanError)?;

    Ok(logs)
}

fn remove_cl_change_entries() -> Result<()> {
    let logs = get_cl_entry_paths()?;

    for log in logs {
        std::fs::remove_file(log)?;
    }

    let gag = Gag::stdout()?;
    remove_empty_subdirs(get_cl_dir()?.as_path())?;
    drop(gag);

    Ok(())
}

fn get_all_changes() -> Result<Vec<Change>> {
    let mut changes: Vec<Change> = get_unreleased_changes()?;
    let logs = get_cl_entry_paths()?;

    for log in logs {
        let mut cl_changes = get_changes(log)?;
        changes.append(&mut cl_changes);
    }

    Ok(changes)
}
