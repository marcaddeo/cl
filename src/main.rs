use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};

fn main() {
    let matches = app_from_crate!()
        .subcommand(
            SubCommand::with_name("added")
                .visible_alias("add")
                .about("Creates a change entry to be placed in the Added section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .index(1)
                        .required(true)
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
                        .index(1)
                        .required(true)
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
                        .index(1)
                        .required(true)
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
                        .index(1)
                        .required(true)
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
                        .index(1)
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("security")
                .about("Creates a change entry to be placed in the Security section of the CHANGELOG")
                .arg(
                    Arg::with_name("description")
                        .help("The description of this change entry")
                        .value_name("DESCRIPTION")
                        .index(1)
                        .required(true)
                )
        )
        .get_matches();

    println!("{:?}", matches);
}
