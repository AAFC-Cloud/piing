use clap::Subcommand;
use eyre::Result;
use std::time::Duration;

use crate::cli::global_args::GlobalArgs;
use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use crate::ping::PingMode;
use crate::runtime;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch the tray application and ping monitors
    Run(RunArgs),
    /// Manage the list of hosts to ping
    #[command(subcommand)]
    Host(HostCommand),
    /// Configure ping mode
    #[command(subcommand)]
    Mode(ModeCommand),
    /// Configure ping interval
    #[command(subcommand)]
    Interval(IntervalCommand),
}

impl Default for Command {
    fn default() -> Self {
        Command::Run(RunArgs {})
    }
}

impl Command {
    pub fn invoke(self, globals: GlobalArgs, dirs: PiingDirs) -> Result<()> {
        match self {
            Command::Run(_args) => runtime::run(globals, dirs),
            Command::Host(cmd) => cmd.invoke(&dirs),
            Command::Mode(cmd) => cmd.invoke(&dirs),
            Command::Interval(cmd) => cmd.invoke(&dirs),
        }
    }
}

#[derive(Debug, Default, clap::Args)]
pub struct RunArgs {}

#[derive(Debug, Subcommand)]
pub enum HostCommand {
    /// Add a host (domain, IP, or URL) to the monitored list
    Add { host: String },
    /// Remove a host from the monitored list
    Remove { host: String },
    /// List the configured hosts
    List,
}

impl HostCommand {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let mut hosts = paths.load_snapshot()?.hosts;
        match self {
            HostCommand::Add { host } => {
                if host.trim().is_empty() {
                    eyre::bail!("Host cannot be empty");
                }
                if !hosts.iter().any(|h| h.eq_ignore_ascii_case(host.trim())) {
                    hosts.push(host.trim().to_string());
                    paths.write_hosts(&hosts)?;
                    println!("Added host: {}", host.trim());
                } else {
                    println!("Host already present: {}", host.trim());
                }
            }
            HostCommand::Remove { host } => {
                let before = hosts.len();
                hosts.retain(|h| !h.eq_ignore_ascii_case(host.trim()));
                if hosts.len() == before {
                    println!("Host not found: {}", host.trim());
                } else {
                    paths.write_hosts(&hosts)?;
                    println!("Removed host: {}", host.trim());
                }
            }
            HostCommand::List => {
                if hosts.is_empty() {
                    println!("No hosts configured.");
                } else {
                    for host in hosts {
                        println!("- {}", host);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum ModeCommand {
    /// Set the ping mode
    Set { mode: PingMode },
    /// Show the current ping mode
    Show,
}

impl ModeCommand {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        match self {
            ModeCommand::Set { mode } => {
                paths.write_mode(mode)?;
                println!("Mode set to {}", mode.as_str());
            }
            ModeCommand::Show => {
                let snapshot = paths.load_snapshot()?;
                println!("Current mode: {}", snapshot.mode.as_str());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum IntervalCommand {
    /// Set the ping interval
    Set {
        #[clap(value_parser = humantime::parse_duration)]
        interval: Duration,
    },
    /// Show the current interval
    Show,
}

impl IntervalCommand {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        match self {
            IntervalCommand::Set { interval } => {
                paths.write_interval(interval)?;
                println!("Interval set to {}", humantime::format_duration(interval));
            }
            IntervalCommand::Show => {
                let snapshot = paths.load_snapshot()?;
                println!(
                    "Current interval: {}",
                    humantime::format_duration(snapshot.interval)
                );
            }
        }
        Ok(())
    }
}
