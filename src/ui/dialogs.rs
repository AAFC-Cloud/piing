use crate::home::PiingDirs;
use eyre::Context;
use eyre::Result;
use std::io::Write;
use std::process::Command;
use teamy_windows::clipboard::write_clipboard;
use teamy_windows::console::console_create;
use teamy_windows::log::LOG_BUFFER;
use teamy_windows::string::EasyPCWSTR;
use tracing::error;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::TASKDIALOG_BUTTON;
use windows::Win32::UI::Controls::TASKDIALOG_COMMON_BUTTON_FLAGS;
use windows::Win32::UI::Controls::TASKDIALOG_FLAGS;
use windows::Win32::UI::Controls::TASKDIALOGCONFIG;
use windows::Win32::UI::Controls::TDF_ALLOW_DIALOG_CANCELLATION;
use windows::Win32::UI::Controls::TDF_POSITION_RELATIVE_TO_WINDOW;
use windows::Win32::UI::Controls::TDF_SIZE_TO_CONTENT;
use windows::Win32::UI::Controls::TaskDialogIndirect;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
const BTN_OK: i32 = 100;
const BTN_OPEN_HOME: i32 = 101;
const BTN_RELOAD: i32 = 102;
const BTN_COPY_MESSAGE: i32 = 103;
const BTN_SHOW_LOGS: i32 = 104;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigDialogChoice {
    Ok,
    OpenHomeDir,
    ReloadNow,
    CopyMessage,
    ShowLogs,
}

/// # Errors
/// Returns an error if the provided operation ultimately fails after user choice
pub fn retry_config_operation<F, T>(
    dirs: &PiingDirs,
    owner: Option<HWND>,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) => {
                error!(?error, "Configuration operation failed");
                loop {
                    let message = format!(
                        "{error}\n\npiing home: {}\nconfig dir: {}\n\nFix the configuration files, then choose 'Reload now' after saving, or use the tray reload action.",
                        dirs.home_dir().display(),
                        dirs.config_dir().display()
                    );
                    match show_config_error_dialog(&message, owner)? {
                        ConfigDialogChoice::Ok => return Err(error),
                        ConfigDialogChoice::OpenHomeDir => {
                            if let Err(open_error) = open_home_directory(dirs) {
                                error!(?open_error, "Failed to open piing home directory");
                            }
                        }
                        ConfigDialogChoice::CopyMessage => {
                            if let Err(copy_error) = write_clipboard(message) {
                                error!(?copy_error, "Failed to copy configuration error text");
                            }
                        }
                        ConfigDialogChoice::ShowLogs => show_logs_console(),
                        ConfigDialogChoice::ReloadNow => break,
                    }
                }
            }
        }
    }
}

/// # Errors
/// Returns an error if dialog creation or display fails
///
/// # Panics
/// Panics if the dialog structure sizes cannot fit into a `u32` (should never happen on supported platforms)
pub fn show_config_error_dialog(
    message: &str,
    owner: Option<HWND>,
) -> eyre::Result<ConfigDialogChoice> {
    let owner_hwnd = owner.unwrap_or_default();
    let window_title = "piing configuration error".easy_pcwstr()?;
    let instruction = "Configuration validation failed".easy_pcwstr()?;
    let content = message.easy_pcwstr()?;
    let ok_label = "OK".easy_pcwstr()?;
    let open_label = "Open home directory".easy_pcwstr()?;
    let copy_label = "Copy message text".easy_pcwstr()?;
    let show_logs_label = "Show logs".easy_pcwstr()?;
    let reload_label = "Reload now".easy_pcwstr()?;

    let buttons = [
        TASKDIALOG_BUTTON {
            nButtonID: BTN_OK,
            pszButtonText: unsafe { ok_label.as_ptr() },
        },
        TASKDIALOG_BUTTON {
            nButtonID: BTN_OPEN_HOME,
            pszButtonText: unsafe { open_label.as_ptr() },
        },
        TASKDIALOG_BUTTON {
            nButtonID: BTN_COPY_MESSAGE,
            pszButtonText: unsafe { copy_label.as_ptr() },
        },
        TASKDIALOG_BUTTON {
            nButtonID: BTN_SHOW_LOGS,
            pszButtonText: unsafe { show_logs_label.as_ptr() },
        },
        TASKDIALOG_BUTTON {
            nButtonID: BTN_RELOAD,
            pszButtonText: unsafe { reload_label.as_ptr() },
        },
    ];

    let config = TASKDIALOGCONFIG {
        cbSize: u32::try_from(std::mem::size_of::<TASKDIALOGCONFIG>())
            .expect("TASKDIALOGCONFIG size fits in u32"),
        hwndParent: owner_hwnd,
        dwFlags: TASKDIALOG_FLAGS(
            TDF_ALLOW_DIALOG_CANCELLATION.0
                | TDF_SIZE_TO_CONTENT.0
                | TDF_POSITION_RELATIVE_TO_WINDOW.0,
        ),
        dwCommonButtons: TASKDIALOG_COMMON_BUTTON_FLAGS(0),
        pszWindowTitle: unsafe { window_title.as_ptr() },
        pszMainInstruction: unsafe { instruction.as_ptr() },
        pszContent: unsafe { content.as_ptr() },
        cButtons: u32::try_from(buttons.len()).expect("button count fits in u32"),
        pButtons: buttons.as_ptr(),
        ..Default::default()
    };

    let mut pressed_button = BTN_OK;
    unsafe {
        if TaskDialogIndirect(&raw const config, Some(&raw mut pressed_button), None, None).is_ok()
        {
            return Ok(match pressed_button {
                BTN_OPEN_HOME => ConfigDialogChoice::OpenHomeDir,
                BTN_COPY_MESSAGE => ConfigDialogChoice::CopyMessage,
                BTN_SHOW_LOGS => ConfigDialogChoice::ShowLogs,
                BTN_RELOAD => ConfigDialogChoice::ReloadNow,
                _ => ConfigDialogChoice::Ok,
            });
        }
    }

    unsafe {
        MessageBoxW(
            Some(owner_hwnd),
            content.as_ref(),
            window_title.as_ref(),
            MB_OK | MB_ICONERROR,
        );
    }
    Ok(ConfigDialogChoice::Ok)
}

fn show_logs_console() {
    if let Err(error) = console_create() {
        error!(?error, "Failed to allocate console for logs");
        return;
    }

    let mut stdout = std::io::stdout();
    if let Err(error) = LOG_BUFFER.replay(&mut stdout) {
        error!(?error, "Failed to replay buffered logs");
    }
    stdout.flush().ok();
}

/// # Errors
/// Returns an error if launching Explorer fails
pub fn open_home_directory(dirs: &PiingDirs) -> Result<()> {
    Command::new("explorer")
        .arg(dirs.home_dir())
        .spawn()
        .wrap_err("Failed to launch explorer")?;
    Ok(())
}
