use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct HostRemoveArgs {
    pub host: String,
}

impl HostRemoveArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let mut hosts = paths.load_snapshot()?.hosts;

        let before = hosts.len();
        hosts.retain(|h| !h.eq_ignore_ascii_case(self.host.trim()));
        if hosts.len() == before {
            println!("Host not found: {}", self.host.trim());
        } else {
            paths.write_hosts(&hosts)?;
            println!("Removed host: {}", self.host.trim());
        }

        Ok(())
    }
}
