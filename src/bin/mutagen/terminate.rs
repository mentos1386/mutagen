use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("terminate")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Stops and removes a synchronization session")
        .usage("mutagen terminate [-h|--help] <session>")
        .arg(Arg::with_name("session")
            .index(1)
            .required(true)
            .help("Specifies the session to terminate"))
}

pub fn main(arguments: &ArgMatches) {
    println!("terminate {:?}", arguments);
}
