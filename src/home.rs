use eyre::Context;
use eyre::Result;
use std::env;
use std::ffi::c_void;
use std::fs::create_dir_all;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::FOLDERID_RoamingAppData;
use windows::Win32::UI::Shell::KF_FLAG_DEFAULT;
use windows::Win32::UI::Shell::SHGetKnownFolderPath;

pub static PIING_HOME: LazyLock<PiingHome> = LazyLock::new(|| {
    let home = PiingHome::new().expect("Failed to identify PIING_HOME");
    home.ensure()
        .expect("Failed to create PIING_HOME and child dirs");
    home
});

#[derive(Debug, Clone)]
pub struct PiingHome(PathBuf);
impl Deref for PiingHome {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Path> for PiingHome {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl PiingHome {
    /// # Errors
    /// Returns an error if determining the home directory fails
    pub fn new() -> Result<Self> {
        if let Some(custom) = env::var_os("PIING_HOME") {
            return Ok(Self(PathBuf::from(custom)));
        }

        let raw_path =
            unsafe { SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None) }
                .wrap_err("Failed to resolve %APPDATA%")?;
        let owned = unsafe { raw_path.to_string() }.wrap_err("Failed to convert path to string")?;
        unsafe { CoTaskMemFree(Some(raw_path.0 as *const c_void)) };
        Ok(Self(PathBuf::from(owned).join("TeamDman").join("piing")))
    }

    /// # Errors
    /// Returns an error if directory creation fails
    pub fn ensure(&self) -> Result<()> {
        create_dir_all(self).wrap_err("Failed to create piing home directory")?;
        create_dir_all(self.logs_dir()).wrap_err("Failed to create piing logs directory")?;
        create_dir_all(self.config_dir()).wrap_err("Failed to create piing config directory")?;
        Ok(())
    }

    #[must_use]
    pub fn logs_dir(&self) -> PathBuf {
        self.0.join("logs")
    }

    #[must_use]
    pub fn config_dir(&self) -> PathBuf {
        self.0.join("config")
    }
}
