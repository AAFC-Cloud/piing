use crate::cli::command::audit::AuditArgs;
use crate::cli::command::home::HomeArgs;
use crate::cli::command::run::RunArgs;
use crate::cli::command::sound::SoundArgs;
use crate::cli::command::target::TargetArgs;
use crate::cli::command::vpn::VpnArgs;
use crate::cli::global_args::GlobalArgs;
use crate::logging::LogWritingBehaviour;
use crate::logging::{self};
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch the tray application and ping monitors
    Run(RunArgs),
    /// Manage the list of targets to ping
    Target(TargetArgs),
    /// Audit log files
    Audit(AuditArgs),
    /// Manage VPN related commands
    Vpn(VpnArgs),
    /// Print the piing home directory
    Home(HomeArgs),
    /// Play or test configured sounds
    Sound(SoundArgs),
}

impl Default for Command {
    fn default() -> Self {
        Command::Run(RunArgs {})
    }
}

impl Command {
    /// # Errors
    /// Returns an error if command execution or logging initialization fails
    pub fn invoke(self, globals: &GlobalArgs) -> Result<()> {
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
        logging::initialize(globals.log_level(), log_behaviour)?;

        match self {
            Command::Run(args) => args.invoke()?,
            Command::Target(args) => args.invoke()?,
            Command::Audit(args) => args.invoke()?,
            Command::Vpn(args) => args.invoke()?,
            Command::Home(args) => args.invoke()?,
            Command::Sound(args) => args.invoke()?,
        }
        Ok(())
    }
}
