use crate::config::VpnCriteria;
use crate::home::PiingDirs;
use chrono::Utc;
use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use teamy_windows::network::NetworkAdapterExt;
use teamy_windows::network::NetworkAdapters;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

#[derive(Debug, Default, Args)]
pub struct AddArgs {}

impl AddArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let adapters = NetworkAdapters::new()?;
        let choices: Vec<Choice<IP_ADAPTER_ADDRESSES_LH>> = adapters
            .iter()
            .map(|adapter| Choice {
                key: format!("{} ({:?})", adapter.display_name(), adapter.id()),
                value: *adapter,
            })
            .collect();
        let picker: PickerTui<IP_ADAPTER_ADDRESSES_LH> = PickerTui::new(choices);
        let selected = picker.pick_many()?;
        let criteria: Vec<_> = selected.into_iter().map(Into::into).collect();
        let vpn_criteria = VpnCriteria(criteria);
        let timestamp = Utc::now().format("%Y-%m-%d_%H%M%S");
        let filename = format!("{timestamp}.piing_hcl");
        let file_path = dirs.vpn_adapter_criteria_dir().join(filename);
        vpn_criteria.write_to_file(file_path)?;
        Ok(())
    }
}
