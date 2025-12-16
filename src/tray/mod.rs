mod window_proc;

use eyre::Context;
use eyre::Result;
use eyre::eyre;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::OnceLock;
use teamy_windows::console::hide_default_console_or_attach_ctrl_handler;
use teamy_windows::event_loop::run_message_loop;
use teamy_windows::hicon::application_icon::get_application_icon;
use teamy_windows::hicon::get_icon_from_current_module;
use teamy_windows::log::LOG_BUFFER;
use teamy_windows::tray::TRAY_ICON_ID;
use teamy_windows::tray::add_tray_icon;
use teamy_windows::window::create_window_for_tray;
use tokio::sync::watch;
use tracing::info;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::NIF_ICON;
use windows::Win32::UI::Shell::NIM_MODIFY;
use windows::Win32::UI::Shell::NOTIFYICONDATAW;
use windows::Win32::UI::Shell::Shell_NotifyIconW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::core::w;

#[derive(Debug)]
pub struct TrayContext {
    pub inherited_console_available: bool,
    pub shutdown_tx: watch::Sender<bool>,
}

static TRAY_WINDOW: OnceLock<isize> = OnceLock::new();
static CURRENT_ICON: OnceLock<Mutex<Option<isize>>> = OnceLock::new();

fn current_icon_slot() -> &'static Mutex<Option<isize>> {
    CURRENT_ICON.get_or_init(|| Mutex::new(None))
}

/// Returns the currently recorded tray icon handle, if any.
///
/// # Panics
/// Panics if the icon tracking mutex is poisoned.
#[must_use]
pub fn current_tray_icon() -> Option<HICON> {
    current_icon_slot()
        .lock()
        .unwrap()
        .map(|bits| HICON(bits as *mut c_void))
}

fn record_tray_window(hwnd: HWND) {
    let _ = TRAY_WINDOW.set(hwnd.0 as isize);
}

fn record_tray_icon(icon: HICON) {
    *current_icon_slot().lock().unwrap() = Some(icon.0 as isize);
}

/// Update the tray icon in place.
/// # Errors
/// Returns an error if the tray window has not been created or the icon cannot be modified.
///
/// # Panics
/// Panics if the NOTIFYICONDATAW structure size does not fit in a u32.
pub fn set_tray_icon(icon: HICON) -> Result<()> {
    record_tray_icon(icon);
    let hwnd_bits = *TRAY_WINDOW
        .get()
        .ok_or_else(|| eyre!("Tray window handle not available"))?;
    let hwnd = HWND(hwnd_bits as *mut c_void);

    let notify_icon_data = NOTIFYICONDATAW {
        cbSize: u32::try_from(std::mem::size_of::<NOTIFYICONDATAW>())
            .expect("NOTIFYICONDATAW size fits in u32"),
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_ICON,
        hIcon: icon,
        ..Default::default()
    };

    unsafe { Shell_NotifyIconW(NIM_MODIFY, &raw const notify_icon_data) }
        .ok()
        .wrap_err("Failed to update tray icon")?;

    Ok(())
}

/// # Errors
/// Returns an error if tray initialization or message loop fails
pub fn run_tray(context: &TrayContext) -> Result<()> {
    let started_with_inherited_console = context.inherited_console_available;
    hide_default_console_or_attach_ctrl_handler()?;

    window_proc::configure(window_proc::TrayWindowConfig {
        inherited_console_available: started_with_inherited_console,
        log_buffer: LOG_BUFFER.clone(),
        shutdown_tx: context.shutdown_tx.clone(),
    })?;

    let window = create_window_for_tray(Some(window_proc::window_proc))?;
    record_tray_window(window);

    let icon = get_icon_from_current_module(w!("green_check_icon")).or_else(|e| {
        tracing::warn!(error = %e, "Failed to load embedded icon, falling back to application icon");
        get_application_icon()
    })?;
    record_tray_icon(icon);
    let tooltip = w!("piing");
    add_tray_icon(window, icon, tooltip)?;

    info!("Tray initialized");

    run_message_loop(None)?;
    Ok(())
}
