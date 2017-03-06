use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("list")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Lists current synchronization sessions")
        .usage("mutagen list [-h|--help] [-m|--monitor] [<session>]")
        .arg(Arg::with_name("monitor")
            .short("m")
            .long("monitor")
            .requires("session")
            .help("Starts dynamic monitoring for a single session"))
        .arg(Arg::with_name("session")
            .index(1)
            .help("Filters listing to a single session"))
}

pub fn main(arguments: &ArgMatches) {
    println!("list {:?}", arguments);
}
