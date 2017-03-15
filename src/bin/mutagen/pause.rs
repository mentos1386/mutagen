use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use mutagen::errors::{Result, ResultExt};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pause")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Pauses a synchronization session")
        .usage("mutagen pause [-h|--help] <session>")
        .arg(Arg::with_name("session")
            .index(1)
            .required(true)
            .help("Specifies the session to pause"))
}

pub fn main(arguments: &ArgMatches) -> Result<()> {
    println!("pause {:?}", arguments);
    bail!("pause not implemented");
}
