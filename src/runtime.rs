use crate::config::ConfigManager;
use crate::config::ConfigSnapshot;
use crate::config::ConfigStore;
use crate::home::PiingDirs;
use crate::ping::parse_destination;
use crate::ping::{self};
use crate::tray;
use eyre::Result;
use std::thread;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::info;
use tracing::warn;

/// # Errors
/// Returns an error if runtime initialization or tray execution fails
pub fn run(dirs: &PiingDirs) -> Result<()> {
    let config_manager = ConfigManager::initialize(dirs)?;

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
            ping_loop(store, client, &mut shutdown_rx).await;
        });
    })
}

async fn ping_loop(
    store: ConfigStore,
    client: reqwest::Client,
    shutdown_rx: &mut watch::Receiver<bool>,
) {
    loop {
        let snapshot = store.snapshot();
        if snapshot.hosts.is_empty() {
            info!("No hosts configured; waiting interval");
        }
        run_snapshot(&client, &snapshot).await;
        let interval = snapshot.interval;
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
}

async fn run_snapshot(client: &reqwest::Client, snapshot: &ConfigSnapshot) {
    for host in &snapshot.hosts {
        let destination = parse_destination(host, snapshot.mode);
        let outcome = ping::execute_ping(client, snapshot.mode, &destination).await;
        log_outcome(&outcome);
    }
}

fn log_outcome(outcome: &ping::PingOutcome) {
    let latency_ms = outcome
        .latency
        .map(|dur| dur.as_secs_f64() * 1000.0)
        .unwrap_or_default();
    if outcome.success {
        info!(
            host = %outcome.host,
            mode = outcome.mode.as_str(),
            success = true,
            latency_ms,
            status = outcome.status.map(|s| s.as_u16()),
            "Ping succeeded"
        );
    } else {
        warn!(
            host = %outcome.host,
            mode = outcome.mode.as_str(),
            success = false,
            error = outcome.error.as_deref().unwrap_or("unknown"),
            "Ping failed"
        );
    }
}
