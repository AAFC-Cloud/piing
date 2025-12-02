mod window_proc;

use crate::config::ConfigManager;
use crate::home::PiingDirs;
use eyre::Result;
use teamy_windows::console::hide_default_console_or_attach_ctrl_handler;
use teamy_windows::event_loop::run_message_loop;
use teamy_windows::hicon::application_icon::get_application_icon;
use teamy_windows::hicon::get_icon_from_current_module;
use teamy_windows::log::LOG_BUFFER;
use teamy_windows::tray::add_tray_icon;
use teamy_windows::window::create_window_for_tray;
use tokio::sync::watch;
use tracing::info;
use windows::core::w;

pub struct TrayContext {
    pub inherited_console_available: bool,
    pub config_manager: ConfigManager,
    pub dirs: PiingDirs,
    pub shutdown_tx: watch::Sender<bool>,
}

pub fn run_tray(context: TrayContext) -> Result<()> {
    let started_with_inherited_console = context.inherited_console_available;
    hide_default_console_or_attach_ctrl_handler()?;

    window_proc::configure(window_proc::TrayWindowConfig {
        inherited_console_available: started_with_inherited_console,
        log_buffer: LOG_BUFFER.clone(),
        config_manager: context.config_manager.clone(),
        dirs: context.dirs.clone(),
        shutdown_tx: context.shutdown_tx.clone(),
    })?;

    let window = create_window_for_tray(Some(window_proc::window_proc))?;

    let icon = get_icon_from_current_module(w!("piing_icon")).or_else(|e| {
        tracing::warn!(error = %e, "Failed to load embedded icon, falling back to application icon");
        get_application_icon()
    })?;
    let tooltip = w!("piing");
    add_tray_icon(window, icon, tooltip)?;

    info!("Tray initialized");

    run_message_loop(None)?;
    Ok(())
}
