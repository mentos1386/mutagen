use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("resume")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("Resumes a synchronization session")
        .usage("mutagen resume [-h|--help] <session>")
        .arg(Arg::with_name("session")
            .index(1)
            .required(true)
            .help("Specifies the session to resume"))
}

pub fn main(arguments: &ArgMatches) {
    println!("resume {:?}", arguments);
}
