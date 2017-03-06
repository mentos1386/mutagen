#[macro_use(crate_version)]
extern crate clap;
extern crate mutagen;

use std::env;

use clap::{App, AppSettings, Arg};

mod create;
mod daemon;
mod list;
mod pause;
mod prompt;
mod resume;
mod terminate;

fn main() {
    // Check if a prompting environment is set. If so, treat this as a prompt
    // request.
    if let Ok(_) = env::var(mutagen::prompt::PROMPTER_ENVIRONMENT_VARIABLE) {
        return prompt::prompt_main();
    }

    // Set up the command line parsing tree. The clap library will automatically
    // add version and help flags, so we just need to add legal.
    let matches = App::new("mutagen")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .about("File synchronization for human beings")
        .version(crate_version!())
        .usage("mutagen [-V|--version] [-h|--help] [-l|--legal] <command> [<args>]")
        .after_help("To see help for a particular command, use 'mutagen <command> --help'")
        .arg(Arg::with_name("legal")
            .short("l")
            .long("legal")
            .help("Prints legal information"))
        .subcommand(create::subcommand().display_order(0))
        .subcommand(list::subcommand().display_order(1))
        .subcommand(pause::subcommand().display_order(2))
        .subcommand(resume::subcommand().display_order(3))
        .subcommand(terminate::subcommand().display_order(4))
        .subcommand(daemon::subcommand().display_order(5))
        .get_matches();

    // Check if we need to print legal information and exit.
    if matches.is_present("legal") {
        println!("{}", mutagen::LEGAL_NOTICE);
        return;
    }

    // Dispatch based on subcommand name. If there's an invalid subcommand name
    // or no subcommand name, then either the argument parser didn't do its job
    // or we didn't handle all flags properly.
    match matches.subcommand_name() {
        Some("create") => create::main(&matches),
        Some("list") => list::main(&matches),
        Some("pause") => pause::main(&matches),
        Some("resume") => resume::main(&matches),
        Some("terminate") => terminate::main(&matches),
        Some("daemon") => daemon::main(&matches),
        _ => panic!("invalid or unhandled subcommand dispatch"),
    }
}
