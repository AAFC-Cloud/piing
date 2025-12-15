use crate::config::ConfigManager;
use crate::home::PiingDirs;
use crate::sound;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum SoundCommand {
    /// Play the configured problem sound once
    Test(SoundTestArgs),
}

impl SoundCommand {
    /// # Errors
    /// Returns an error if the test fails to play the configured sound
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self {
            SoundCommand::Test(args) => args.invoke(dirs),
        }
    }
}

#[derive(Debug, Default, Args)]
pub struct SoundTestArgs {}

impl SoundTestArgs {
    /// # Errors
    /// Returns an error if loading config or playing the sound fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let config_manager = ConfigManager::initialize(dirs)?;
        let snapshot = config_manager.store.snapshot();
        let sound_cfg = snapshot.problem_sound();
        println!("Playing configured problem sound: {}", sound_cfg.path().display());
        // Block until playback completes so the CLI command actually plays
        // the sound before exiting.
        sound::play_problem_sound_blocking(sound_cfg)?;
        println!("Playback completed");
        Ok(())
    }
}
