use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct HostListArgs {}

impl HostListArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let hosts = paths.load_snapshot()?.hosts;

        if hosts.is_empty() {
            println!("No hosts configured.");
        } else {
            for host in hosts {
                println!("{}", host);
            }
        }

        Ok(())
    }
}
