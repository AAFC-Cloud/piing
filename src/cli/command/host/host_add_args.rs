use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct HostAddArgs {
    pub host: String,
}

impl HostAddArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let mut hosts = paths.load_snapshot()?.hosts;

        if self.host.trim().is_empty() {
            eyre::bail!("Host cannot be empty");
        }
        if !hosts
            .iter()
            .any(|h| h.eq_ignore_ascii_case(self.host.trim()))
        {
            hosts.push(self.host.trim().to_string());
            paths.write_hosts(&hosts)?;
            println!("Added host: {}", self.host.trim());
        } else {
            println!("Host already present: {}", self.host.trim());
        }

        Ok(())
    }
}
