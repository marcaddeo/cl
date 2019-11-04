use anyhow::Result;
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, AppSettings, Arg,
    SubCommand,
};
use clparse::changelog::{Change, ReleaseBuilder};
use err_derive::Error;
use scan_dir::ScanDir;
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
        .get_matches();

    match matches.subcommand() {
        ("", None) => {
            let changes = get_all_changes()?;
            let output = match matches.value_of("format").unwrap() {
                "json" => {
                    format!("{}", serde_json::to_string_pretty(&changes)?)
                }
                "yaml" | "yml" => {
                    let mut output = serde_yaml::to_string(&changes)?;
                    if matches.is_present("no-headings") {
                        output = output.replace("---\n", "");
                    }
                    format!("{}", output.to_string().trim_end())
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
                    format!("{}", output.to_string().trim_end())
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
    cl_path.set_extension("yml");

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

fn get_all_changes() -> Result<Vec<Change>> {
    let mut changes: Vec<Change> = Vec::new();
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

    for log in logs {
        let mut cl_changes = get_changes(log)?;
        changes.append(&mut cl_changes);
    }

    Ok(changes)
}
