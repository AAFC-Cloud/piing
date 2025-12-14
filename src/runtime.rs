use crate::cli::command::vpn::check::CheckArgs;
use crate::config::ConfigManager;
use crate::config::ConfigSnapshot;
use crate::config::ConfigStore;
use crate::config::targets::Target;
use crate::home::PiingDirs;
use crate::ping::PingOutcome;
use crate::ping::{self};
use crate::tray;
use crate::ui::dialogs::retry_config_operation;
use eyre::Result;
use std::thread;
use std::time::Duration;
use teamy_windows::hicon::get_icon_from_current_module;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::error;
use tracing::info;
use tracing::warn;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::core::w;

/// # Errors
/// Returns an error if runtime initialization or tray execution fails
pub fn run(dirs: &PiingDirs) -> Result<()> {
    let config_manager = retry_config_operation(dirs, None, || ConfigManager::initialize(dirs))?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let ping_store = config_manager.store.clone();
    let worker_handle = spawn_ping_runtime(ping_store, shutdown_rx);

    let tray_context = tray::TrayContext {
        inherited_console_available: teamy_windows::console::is_inheriting_console(),
        config_manager: config_manager.clone(),
        dirs: dirs.clone(),
        shutdown_tx: shutdown_tx.clone(),
    };
    tray::run_tray(&tray_context)?;

    let _ = shutdown_tx.send(true);
    worker_handle.join().ok();
    Ok(())
}

fn spawn_ping_runtime(
    store: ConfigStore,
    mut shutdown_rx: watch::Receiver<bool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");
        runtime.block_on(async move {
            let client = match ping::build_http_client() {
                Ok(client) => client,
                Err(error) => {
                    warn!("Failed to build HTTP client: {error}");
                    return;
                }
            };
            if let Err(e) = ping_loop(store, client, &mut shutdown_rx).await {
                error!("Ping runtime encountered an error: {e}");
            }
        });
    })
}

const DEFAULT_INTERVAL: Duration = Duration::from_secs(1);

async fn ping_loop(
    config_store: ConfigStore,
    client: reqwest::Client,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> eyre::Result<()> {
    let success_icon = get_icon_from_current_module(w!("green_check_icon"))?;
    let failure_icon = get_icon_from_current_module(w!("red_x_icon"))?;

    loop {
        let ConfigSnapshot {
            targets,
            vpn_criteria,
            ..
        } = config_store.snapshot();

        if targets.is_empty() {
            info!("No targets configured; waiting interval");
        }

        let vpn_active = CheckArgs { quiet: true }
            .invoke(&vpn_criteria)
            .unwrap_or(false);

        if !targets.is_empty() {
            let outcomes = run_targets(&client, &targets, vpn_active).await;
            apply_tray_icon(&outcomes, success_icon, failure_icon);
        }

        let interval = targets
            .iter()
            .map(|target| target.interval)
            .min()
            .unwrap_or(DEFAULT_INTERVAL);
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Ping loop shutting down");
                    break;
                }
            }
            () = sleep(interval) => {}
        }
    }
    Ok(())
}

async fn run_targets(
    client: &reqwest::Client,
    targets: &[Target],
    vpn_active: bool,
) -> Vec<PingOutcome> {
    let mut join_set = JoinSet::new();

    for target in targets {
        let mode = target.mode;
        let destination = target.value.clone();
        let client = client.clone();
        join_set.spawn(async move {
            let outcome = ping::execute_ping(&client, mode, &destination).await;
            log_outcome(&outcome, vpn_active);
            outcome
        });
    }

    let mut outcomes = Vec::with_capacity(targets.len());
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(outcome) => outcomes.push(outcome),
            Err(error) => error!("Ping task failed: {error}"),
        }
    }

    outcomes
}

fn log_outcome(outcome: &PingOutcome, vpn_active: bool) {
    let latency_ms = outcome
        .latency
        .map(|dur| dur.as_millis())
        .unwrap_or_default();
    if outcome.success {
        info!(
            host = %outcome.host,
            mode = outcome.mode.as_str(),
            success = true,
            latency_ms,
            status = outcome.status.map(|s| s.as_u16()),
            vpn_active,
            "Ping succeeded"
        );
    } else {
        warn!(
            host = %outcome.host,
            mode = outcome.mode.as_str(),
            success = false,
            error = outcome.error.as_deref().unwrap_or("unknown"),
            vpn_active,
            "Ping failed"
        );
    }
}

fn apply_tray_icon(outcomes: &[PingOutcome], success_icon: HICON, failure_icon: HICON) {
    if outcomes.is_empty() {
        return;
    }

    let is_success = outcomes.iter().all(|outcome| outcome.success);
    let icon = if is_success {
        success_icon
    } else {
        failure_icon
    };
    if let Err(e) = tray::set_tray_icon(icon) {
        warn!("Failed to set tray icon: {e:?}");
    }
}
