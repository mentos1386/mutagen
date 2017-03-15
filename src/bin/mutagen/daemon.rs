use std::env;
use std::process::Command;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use mutagen::errors::{Result, ResultExt};
use mutagen::process::CommandExt;

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("daemon")
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

pub fn main(arguments: &ArgMatches) -> Result<()> {
    // If stopping is requested, try to send a termination request.
    if arguments.is_present("stop") {
        // TODO: Implement.
        unimplemented!();
    }

    // Unless foreground running has been requested, restart in the background.
    if !arguments.is_present("run") {
        // Compute the path to the current process.
        let mutagen_path = env::current_exe()
                          .chain_err(|| "unable to get Mutagen binary path")?;

        // Set up the daemon process to run in the background. The
        // std::process::Child object doesn't kill on drop, so we don't need to
        // hold onto it.
        Command::new(&mutagen_path).arg("daemon").arg("--run")
            .background(true)
            .spawn()
            .chain_err(|| "unable to spawn daemon process")?;

        // Success.
        return Ok(());
    }

    // TODO: Acquire the daemon lock and ensure its release when we're done.

    // TODO: Start the IPC server.

    // TODO: Watch for termination from a signal, an internal error, or a client
    // request.
    println!("daemon {:?}", arguments);
    bail!("daemon not implemented");
}
