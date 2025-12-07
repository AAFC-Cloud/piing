use crate::cli::command::audit::AuditArgs;
use crate::cli::command::host::HostArgs;
use crate::cli::command::interval::IntervalArgs;
use crate::cli::command::mode::ModeArgs;
use crate::cli::command::run::RunArgs;
use crate::cli::global_args::GlobalArgs;
use crate::home::PiingDirs;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch the tray application and ping monitors
    Run(RunArgs),
    /// Manage the list of hosts to ping
    Host(HostArgs),
    /// Configure ping mode
    Mode(ModeArgs),
    /// Configure ping interval
    Interval(IntervalArgs),
    /// Audit log files
    Audit(AuditArgs),
}

impl Default for Command {
    fn default() -> Self {
        Command::Run(RunArgs {})
    }
}

impl Command {
    /// # Errors
    /// Returns an error if command execution or logging initialization fails
    pub fn invoke(self, globals: GlobalArgs, dirs: &PiingDirs) -> Result<()> {
        use crate::logging::LogWritingBehaviour;
        use crate::logging::{self};

        // Determine logging behavior based on command and global args
        let log_behaviour = match &self {
            Command::Run(_) => {
                // Run command always writes to file
                if let Some(ref path) = globals.log_file {
                    LogWritingBehaviour::TerminalAndSpecificFile(path.clone())
                } else {
                    LogWritingBehaviour::TerminalAndDefaultFile
                }
            }
            _ => {
                // Other commands only write to file if --log-file is specified
                if let Some(ref path) = globals.log_file {
                    LogWritingBehaviour::TerminalAndSpecificFile(path.clone())
                } else {
                    LogWritingBehaviour::TerminalOnly
                }
            }
        };

        // Initialize logging for all commands
        logging::initialize(globals.log_level(), dirs, log_behaviour)?;

        match self {
            Command::Run(args) => args.invoke(globals, dirs),
            Command::Host(args) => args.invoke(dirs),
            Command::Mode(args) => args.invoke(dirs),
            Command::Interval(args) => args.invoke(dirs),
            Command::Audit(args) => args.invoke(dirs),
        }
    }
}
