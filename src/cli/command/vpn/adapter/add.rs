use crate::config::ConfigPaths;
use crate::config::targets::sanitize_label;
use crate::config::vpn_criterion::VpnCriterion;
use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use hcl::edit::structure::Body;
use std::collections::HashSet;
use teamy_windows::network::NetworkAdapterExt;
use teamy_windows::network::NetworkAdapters;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

#[derive(Debug, Default, Args)]
pub struct AddArgs {}

impl AddArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, paths: &ConfigPaths) -> Result<()> {
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
        if selected.is_empty() {
            println!("No adapters selected; nothing to write.");
            return Ok(());
        }

        let mut used_labels = HashSet::new();
        let mut body = Body::builder();
        for (index, adapter) in selected.into_iter().enumerate() {
            let criterion: VpnCriterion = adapter.into();
            let preferred = criterion
                .display_name
                .clone()
                .unwrap_or_else(|| format!("vpn_{index}"));
            let mut label = sanitize_label(&preferred);
            if label.is_empty() {
                label = format!("vpn_{index}");
            }
            while !used_labels.insert(label.clone()) {
                label.push('_');
            }
            body = body.block(criterion.block(&label));
        }

        let body = body.build();
        let file_path = paths.unique_file_path("vpn");
        paths.write_body(&file_path, &body)?;
        println!("Saved VPN adapter criteria to {}", file_path.display());
        Ok(())
    }
}
