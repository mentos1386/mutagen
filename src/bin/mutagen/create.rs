use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use mutagen::errors::Result;

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Starts a new synchronization session")
        .usage("mutagen create [-h|--help] [-i|--ignore <pattern>] <alpha> <beta>")
        .arg(Arg::with_name("ignore")
            .short("i")
            .long("ignore")
            .number_of_values(1)
            .multiple(true)
            .value_name("pattern")
            .help("Adds an ignore pattern to the session"))
        .arg(Arg::with_name("alpha")
            .index(1)
            .required(true)
            .help("Specifies the alpha endpoint URL"))
        .arg(Arg::with_name("beta")
            .index(2)
            .required(true)
            .help("Specifies the beta endpoint URL"))
}

pub fn main(arguments: &ArgMatches) -> Result<()> {
    println!("create {:?}", arguments);
    bail!("create not implemented");
}
