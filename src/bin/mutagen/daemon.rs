use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("daemon")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Starts and controls the synchronization daemon lifecycle")
        .usage("mutagen daemon [-h|--help] [-s|--stop]")
        .arg(Arg::with_name("run")
            .short("r")
            .long("run")
            .hidden(true)
            .help("Runs the daemon (instead of forking)"))
        .arg(Arg::with_name("stop")
            .short("s")
            .long("stop")
            .conflicts_with("run")
            .help("Stops any existing daemon instance"))
}

pub fn main(arguments: &ArgMatches) {
    println!("daemon {:?}", arguments);
}
