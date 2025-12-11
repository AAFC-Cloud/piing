use crate::cli::command::audit::AuditArgs;
use crate::config::ConfigManager;
use crate::home::PiingDirs;
use crate::ui::dialogs::retry_config_operation;
use eyre::Result;
use eyre::eyre;
use std::io::Write;
use std::process::Command;
use std::sync::OnceLock;
use teamy_windows::console::console_attach;
use teamy_windows::console::console_create;
use teamy_windows::console::console_detach;
use teamy_windows::log::BufferSink;
use teamy_windows::tray::WM_TASKBAR_CREATED;
use teamy_windows::tray::WM_USER_TRAY_CALLBACK;
use teamy_windows::tray::delete_tray_icon;
use teamy_windows::tray::re_add_tray_icon;
use tracing::error;
use tracing::info;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::POINT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::Console::ATTACH_PARENT_PROCESS;
use windows::Win32::UI::WindowsAndMessaging::AppendMenuW;
use windows::Win32::UI::WindowsAndMessaging::CreatePopupMenu;
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::Win32::UI::WindowsAndMessaging::DestroyMenu;
use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
use windows::Win32::UI::WindowsAndMessaging::EnableMenuItem;
use windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::MF_BYCOMMAND;
use windows::Win32::UI::WindowsAndMessaging::MF_GRAYED;
use windows::Win32::UI::WindowsAndMessaging::MF_SEPARATOR;
use windows::Win32::UI::WindowsAndMessaging::MF_STRING;
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::TPM_LEFTALIGN;
use windows::Win32::UI::WindowsAndMessaging::TPM_RETURNCMD;
use windows::Win32::UI::WindowsAndMessaging::TPM_RIGHTBUTTON;
use windows::Win32::UI::WindowsAndMessaging::TPM_TOPALIGN;
use windows::Win32::UI::WindowsAndMessaging::TrackPopupMenu;
use windows::Win32::UI::WindowsAndMessaging::WM_CLOSE;
use windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU;
use windows::Win32::UI::WindowsAndMessaging::WM_CREATE;
use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONDBLCLK;
use windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONUP;
use windows::core::PCWSTR;
use windows::core::w;

type ShutdownSender = tokio::sync::watch::Sender<bool>;

const CMD_SHOW_LOGS: usize = 0x2000;
const CMD_HIDE_LOGS: usize = 0x2001;
const CMD_OPEN_HOME: usize = 0x2002;
const CMD_RELOAD_CONFIG: usize = 0x2003;
const CMD_AUDIT: usize = 0x2004;
const CMD_EXIT_APP: usize = 0x2005;

#[derive(Clone)]
pub struct TrayWindowConfig {
    pub inherited_console_available: bool,
    pub log_buffer: BufferSink,
    pub config_manager: ConfigManager,
    pub dirs: PiingDirs,
    pub shutdown_tx: ShutdownSender,
}

static CONFIG: OnceLock<TrayWindowConfig> = OnceLock::new();

pub fn configure(config: TrayWindowConfig) -> Result<()> {
    CONFIG
        .set(config)
        .map_err(|_| eyre!("Tray window already configured"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConsoleMode {
    Detached,
    Inherited,
    Owned,
}

struct TrayWindowState {
    console_mode: ConsoleMode,
    inherited_console_available: bool,
    log_buffer: BufferSink,
    config_manager: ConfigManager,
    dirs: PiingDirs,
    shutdown_tx: ShutdownSender,
}

impl TrayWindowState {
    fn new(config: &TrayWindowConfig) -> Self {
        let console_mode = if config.inherited_console_available {
            ConsoleMode::Inherited
        } else {
            ConsoleMode::Detached
        };
        Self {
            console_mode,
            inherited_console_available: config.inherited_console_available,
            log_buffer: config.log_buffer.clone(),
            config_manager: config.config_manager.clone(),
            dirs: config.dirs.clone(),
            shutdown_tx: config.shutdown_tx.clone(),
        }
    }

    fn can_show_logs(&self) -> bool {
        self.console_mode != ConsoleMode::Owned
    }

    fn can_hide_logs(&self) -> bool {
        self.console_mode == ConsoleMode::Owned
    }

    fn show_logs(&mut self) {
        if !self.can_show_logs() {
            return;
        }
        if self.console_mode == ConsoleMode::Inherited
            && let Err(error) = console_detach()
        {
            error!("Failed to detach console: {error}");
            return;
        }
        if let Err(error) = console_create() {
            error!("Failed to allocate console: {error}");
            return;
        }
        if let Err(error) = self.replay_buffer() {
            error!("Failed to replay logs: {error}");
        }
        self.console_mode = ConsoleMode::Owned;
        info!("Console window allocated for logs");
    }

    fn hide_logs(&mut self) {
        if !self.can_hide_logs() {
            return;
        }
        if let Err(error) = console_detach() {
            error!("Failed to detach console: {error}");
            return;
        }
        if self.inherited_console_available {
            if let Err(error) = console_attach(ATTACH_PARENT_PROCESS) {
                error!("Failed to reattach to parent console: {error}");
                self.console_mode = ConsoleMode::Detached;
            } else {
                self.console_mode = ConsoleMode::Inherited;
            }
        } else {
            self.console_mode = ConsoleMode::Detached;
        }
    }

    fn replay_buffer(&self) -> Result<()> {
        let mut stdout = std::io::stdout();
        self.log_buffer.replay(&mut stdout)?;
        stdout.flush().ok();
        Ok(())
    }

    fn open_home_folder(&self) {
        if let Err(error) = Command::new("explorer").arg(self.dirs.home_dir()).spawn() {
            error!("Failed to open home folder: {error}");
        }
    }

    fn reload_config(&self, owner: HWND) {
        match retry_config_operation(&self.dirs, Some(owner), || self.config_manager.reload()) {
            Ok(snapshot) => {
                let target_count = snapshot.targets().len();
                info!(targets = target_count, "Configuration reloaded");
            }
            Err(error) => error!("Configuration reload aborted: {error}"),
        }
    }

    fn run_audit(&mut self) {
        // Ensure we have a console to show output
        if self.console_mode != ConsoleMode::Owned {
            self.show_logs();
        }

        // Run audit
        match AuditArgs::default().invoke(&self.dirs) {
            Ok(()) => info!("Audit completed"),
            Err(error) => error!("Audit failed: {error}"),
        }
    }

    fn request_exit(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    fn show_context_menu(&mut self, hwnd: HWND) {
        let _ = unsafe { SetForegroundWindow(hwnd) }.ok();
        let menu = match unsafe { CreatePopupMenu() } {
            Ok(menu) => menu,
            Err(error) => {
                error!("Failed to create context menu: {error}");
                return;
            }
        };

        unsafe { AppendMenuW(menu, MF_STRING, CMD_SHOW_LOGS, w!("Show logs")) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_HIDE_LOGS, w!("Hide logs")) }.ok();
        unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_OPEN_HOME, w!("Open home folder")) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_RELOAD_CONFIG, w!("Reload config")) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_AUDIT, w!("Audit")) }.ok();
        unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_EXIT_APP, w!("Exit")) }.ok();

        if !self.can_show_logs() {
            let _ = unsafe {
                EnableMenuItem(
                    menu,
                    CMD_SHOW_LOGS.try_into().expect("CMD_SHOW_LOGS fits in u32"),
                    MF_BYCOMMAND | MF_GRAYED,
                )
            };
        }
        if !self.can_hide_logs() {
            let _ = unsafe {
                EnableMenuItem(
                    menu,
                    CMD_HIDE_LOGS.try_into().expect("CMD_HIDE_LOGS fits in u32"),
                    MF_BYCOMMAND | MF_GRAYED,
                )
            };
        }

        let mut cursor_pos = POINT::default();
        unsafe { GetCursorPos(&raw mut cursor_pos) }.ok();
        #[allow(clippy::cast_sign_loss)]
        let selection = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD,
                cursor_pos.x,
                cursor_pos.y,
                None,
                hwnd,
                None,
            )
        }
        .0;

        unsafe { DestroyMenu(menu) }.ok();

        #[allow(clippy::cast_sign_loss)]
        match selection as usize {
            CMD_SHOW_LOGS => self.show_logs(),
            CMD_HIDE_LOGS => self.hide_logs(),
            CMD_OPEN_HOME => self.open_home_folder(),
            CMD_RELOAD_CONFIG => self.reload_config(hwnd),
            CMD_AUDIT => self.run_audit(),
            CMD_EXIT_APP => {
                self.request_exit();
                unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }.ok();
            }
            _ => {}
        }
    }
}

fn store_state(hwnd: HWND, state: Box<TrayWindowState>) {
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize) };
}

fn with_state(hwnd: HWND, action: impl FnOnce(&mut TrayWindowState)) {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return;
    }
    let state = unsafe { &mut *(ptr as *mut TrayWindowState) };
    action(state);
}

fn drop_state(hwnd: HWND) {
    let ptr = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut TrayWindowState)) };
    }
}

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            if let Some(config) = CONFIG.get() {
                store_state(hwnd, Box::new(TrayWindowState::new(config)));
                LRESULT(0)
            } else {
                error!("Tray config missing");
                LRESULT(-1)
            }
        }
        WM_USER_TRAY_CALLBACK => {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            match lparam.0 as u32 {
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    with_state(hwnd, |state| state.show_context_menu(hwnd));
                }
                WM_LBUTTONDBLCLK => with_state(hwnd, TrayWindowState::show_logs),
                _ => {}
            }
            LRESULT(0)
        }
        m if m == *WM_TASKBAR_CREATED => {
            if let Err(error) = re_add_tray_icon() {
                error!("Failed to re-add tray icon: {error}");
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) }.ok();
            LRESULT(0)
        }
        WM_DESTROY => {
            if let Err(error) = delete_tray_icon(hwnd) {
                error!("Failed to delete tray icon: {error}");
            }
            with_state(hwnd, |state| {
                state.request_exit();
            });
            drop_state(hwnd);
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}
