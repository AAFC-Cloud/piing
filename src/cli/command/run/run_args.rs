use crate::cli::global_args::GlobalArgs;
use crate::home::PiingDirs;
use crate::runtime;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct RunArgs {}

impl RunArgs {
    pub fn invoke(self, _globals: GlobalArgs, dirs: PiingDirs) -> Result<()> {
        runtime::run(dirs)
    }
}
