use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex, Condvar};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use ctrlc::set_handler as set_termination_handler;

use mutagen::daemon::Lock;
use mutagen::errors::{Result, ResultExt};
use mutagen::process::CommandExt;

#[derive(PartialEq)]
enum TerminationStatus {
    Unterminated,
    TerminatedBySignal,
    TerminatedByClient,
    TerminatedByError,
}

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

    // Acquire the daemon lock. This lock will be released when dropped.
    // HACK: Rust complains that this variable is unused, but this warning can
    // be squelched by prefixing its name with an underscore. It's not clear if
    // this is the idiomatic way of handling this for scoped locks taking
    // advantage of RAII-style behavior.
    let _lock = Lock::acquire().chain_err(|| "unable to acquire daemon lock")?;

    // TODO: Create and start the server.

    // Create a termination condition.
    let termination = Arc::new((
        Mutex::new(TerminationStatus::Unterminated),
        Condvar::new()
    ));

    // Monitor for termination from signals.
    // HACK: The ctrlc crate's Error type doesn't implement std::error::Error,
    // so we can't chain it.
    let signal_termination = termination.clone();
    let handler_result = set_termination_handler(move || {
        let &(ref lock, ref cvar) = &*signal_termination;
        let mut terminated = lock.lock().unwrap();
        if *terminated == TerminationStatus::Unterminated {
            *terminated = TerminationStatus::TerminatedBySignal;
        }
        cvar.notify_one();
    });
    if !handler_result.is_ok() {
        bail!("unable to set signal handler");
    }

    // TODO: Monitor for termination from the server, either due to an error or
    // a client-initiated termination.

    // Wait for termination.
    let &(ref lock, ref cvar) = &*termination;
    let mut terminated = lock.lock().unwrap();
    while *terminated == TerminationStatus::Unterminated {
        terminated = cvar.wait(terminated).unwrap();
    }

    // TODO: Initiate clean shutdown of the server. Maybe just have it implement
    // drop.

    // TODO: Handle the result based on termination reason.
    Ok(())
}
