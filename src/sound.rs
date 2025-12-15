use crate::config::ProblemSound;
use std::sync::Arc;
use eyre::Result;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use tracing::warn;
use tracing::debug;
use windows::Win32::Media::Multimedia::mciGetErrorStringW;
use windows::Win32::Media::Multimedia::mciSendStringW;
use windows::Win32::Media::Audio::waveOutSetVolume;
use windows::Win32::Media::Audio::HWAVEOUT;
use windows::core::PCWSTR;

static ALIAS_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Play the configured problem sound on a background thread.
///
/// # Errors
/// Returns an error if the sound cannot be scheduled for playback.
pub fn play_problem_sound(sound: Arc<ProblemSound>) -> Result<()> {
    // Clone the Arc for the background thread; cloning an Arc is cheap and
    // allows the caller to retain their reference while the playback runs.
    let sound = sound.clone();
    debug!(?sound, "Scheduling problem sound for background playback");
    thread::spawn(move || {
        if let Err(error) = play_sound_internal(sound.path(), sound.volume()) {
            warn!(?error, path = %sound.path().display(), "Failed to play problem sound");
        }
    });
    Ok(())
}

/// Play the configured problem sound synchronously and wait until playback
/// completes. This is useful for CLI commands that should not return before
/// the sound has finished.
pub fn play_problem_sound_blocking(sound: Arc<ProblemSound>) -> Result<()> {
    play_sound_internal(sound.path(), sound.volume())
}

/// Note that MCI is a legacy feature and has been superseded by MediaPlayer.
/// <https://learn.microsoft.com/en-us/windows/win32/multimedia/the-wait-notify-and-test-flags>
/// <https://microsoft.github.io/windows-rs/features/#/63/search/MediaPlayer>
fn play_sound_internal(path: &Path, volume: f32) -> Result<()> {
    if !path.exists() {
        return Err(eyre::eyre!(
            "Problem sound path does not exist: {}",
            path.display()
        ));
    }

    let alias_id = ALIAS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let alias = make_alias(alias_id);

    debug!(path = %path.display(), volume, alias = %alias, "Starting playback");

    // Try opening without an explicit type first (works for many formats).
    let open_cmd = build_open_cmd(path, &alias);
    match send_mci(&open_cmd) {
        Ok(()) => {
            debug!(command = %open_cmd, "Opened media without explicit type");
        }
        Err(orig_err) => {
            debug!(command = %open_cmd, error = %orig_err, "Open without type failed; attempting type-specific open");
            if let Some(mci_type) = infer_mci_type(path) {
                let open_with_type = build_open_cmd_with_type(path, &alias, mci_type);
                debug!(command = %open_with_type, mci_type, "Retrying open with explicit type");
                // If this fails, propagate the error to the caller.
                send_mci(&open_with_type)?;
            } else {
                return Err(orig_err);
            }
        }
    }

    let volume_value = compute_volume_value(volume);
    // If this is a wave audio file, try to set device volume via waveOutSetVolume
    // which some drivers support even when `setaudio` fails.
    if let Some(mci_type) = infer_mci_type(path) {
        if mci_type == "waveaudio" {
            match try_set_waveout_volume(volume_value) {
                Ok(()) => {
                    debug!(path = %path.display(), volume_value, "waveOutSetVolume succeeded; skipping setaudio");
                    let play_cmd = build_play_cmd(&alias);
                    let res = send_mci(&play_cmd);
                    let close_cmd = build_close_cmd(&alias);
                    let _ = send_mci(&close_cmd);
                    return res;
                }
                Err(err) => {
                    debug!(path = %path.display(), error = %err, "waveOutSetVolume failed; falling back to setaudio variants");
                }
            }
        }
    }
    let set_cmds = build_set_cmd_variants(&alias, volume_value);
    let play_cmd = build_play_cmd(&alias);

    // Try each `setaudio` variant until one succeeds, then play. If none
    // succeed, attempt playback anyway (with default/system volume).
    let mut set_succeeded = false;
    for cmd in &set_cmds {
        match send_mci(cmd) {
            Ok(()) => {
                debug!(command = %cmd, "setaudio variant succeeded");
                set_succeeded = true;
                break;
            }
            Err(e) => {
                debug!(command = %cmd, error = %e, "setaudio variant failed");
            }
        }
    }

    let result = if set_succeeded {
        // Play normally if set succeeded.
        send_mci(&play_cmd)
    } else {
        debug!("All setaudio variants failed; attempting playback without setting volume");
        match send_mci(&play_cmd) {
            Ok(()) => {
                warn!("Failed to set volume; playback started with default volume");
                Ok(())
            }
            Err(play_err) => {
                // Play failed; attempt explicit reopen+retry as before.
                debug!(command = %play_cmd, error = %play_err, "play failed after setaudio variants failed; attempting reopen with explicit type");
                if let Some(mci_type) = infer_mci_type(path) {
                    let open_with_type = build_open_cmd_with_type(path, &alias, mci_type);
                    debug!(command = %open_with_type, mci_type, "Reopening with explicit type and retrying set/play");
                    // Close previous alias, ignore close error
                    let _ = send_mci(&build_close_cmd(&alias));
                    send_mci(&open_with_type)?;
                    // Retry set attempts
                    for cmd in &set_cmds {
                        if send_mci(cmd).is_ok() {
                            break;
                        }
                    }
                    send_mci(&play_cmd)
                } else {
                    Err(play_err)
                }
            }
        }
    };

    let close_cmd = build_close_cmd(&alias);
    let _ = send_mci(&close_cmd);

    result
}

// Helpers for building MCI commands. Extracted so behaviour is testable and so
// the driver-compatibility fix (no fixed `type`) can be validated.
fn make_alias(alias_id: usize) -> String {
    format!("piing_problem_sound_{alias_id}")
}

fn build_open_cmd(path: &Path, alias: &str) -> String {
    format!("open \"{}\" alias {alias}", path.to_string_lossy())
}

fn build_open_cmd_with_type(path: &Path, alias: &str, mci_type: &str) -> String {
    format!("open \"{}\" type {mci_type} alias {alias}", path.to_string_lossy())
}

fn infer_mci_type(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .and_then(|ext| match ext.as_str() {
            "wav" => Some("waveaudio"),
            "mp3" => Some("mpegvideo"),
            "wma" => Some("mpegvideo"),
            _ => None,
        })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn compute_volume_value(volume: f32) -> u32 {
    let clamped = volume.clamp(0.0, 1.0);
    (clamped * 1000.0).round() as u32
}

fn build_set_cmd(alias: &str, volume_value: u32) -> String {
    // Prefer the explicit `output` modifier which many drivers expect when
    // setting playback volume. Fall back to the legacy form without
    // `output` if the driver does not recognize the command.
    format!("setaudio {alias} output volume to {volume_value}")
}

fn build_set_cmd_variants(alias: &str, volume_value: u32) -> Vec<String> {
    vec![
        build_set_cmd(alias, volume_value),
        // Fallback for drivers that accept the shorter form.
        format!("setaudio {alias} volume to {volume_value}"),
    ]
}

fn build_play_cmd(alias: &str) -> String {
    format!("play {alias} from 0 wait")
}

fn build_close_cmd(alias: &str) -> String {
    format!("close {alias}")
}

fn try_set_waveout_volume(volume_value: u32) -> Result<()> {
    // MCI uses 0..1000 for volume; Windows waveOut uses 0..0xFFFF per channel.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let scaled = ((volume_value as f64 / 1000.0) * 0xFFFF as f64).round() as u32;
    let dw_volume = (scaled << 16) | (scaled & 0xFFFF);
    // Use the WAVE_MAPPER device (represented by HWAVEOUT(0) here).
    // Passing None uses the WAVE_MAPPER (default) device.
    let res = unsafe { waveOutSetVolume(None, dw_volume) };
    if res != 0 {
        Err(eyre::eyre!("waveOutSetVolume failed with code {}", res))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn open_cmd_does_not_force_waveaudio() {
        let path = PathBuf::from(r"C:\sounds\alert.mp3");
        let alias = make_alias(1);
        let cmd = build_open_cmd(&path, &alias);
        assert_eq!(
            cmd,
            "open \"C:\\sounds\\alert.mp3\" alias piing_problem_sound_1"
        );
        assert!(!cmd.contains("waveaudio"));
    }

    #[test]
    fn volume_scaling_and_set_cmd() {
        assert_eq!(compute_volume_value(0.0), 0);
        assert_eq!(compute_volume_value(1.0), 1000);
        assert_eq!(compute_volume_value(0.5), 500);
        // rounding behaviour
        assert_eq!(compute_volume_value(0.1234), 123);

        let set = build_set_cmd("piing_problem_sound_1", 123);
        assert_eq!(set, "setaudio piing_problem_sound_1 output volume to 123");

        let variants = build_set_cmd_variants("piing_problem_sound_1", 123);
        assert_eq!(variants[0], "setaudio piing_problem_sound_1 output volume to 123");
        assert_eq!(variants[1], "setaudio piing_problem_sound_1 volume to 123");
    }

    #[test]
    fn play_and_close_cmds() {
        assert_eq!(build_play_cmd("a"), "play a from 0 wait");
        assert_eq!(build_close_cmd("a"), "close a");
    }

    #[test]
    fn blocking_play_fails_for_missing_file() {
        let sound = Arc::new(crate::config::ProblemSound::new(PathBuf::from(r"C:\nope\missing.wav"), 1.0));
        let res = play_problem_sound_blocking(sound);
        assert!(res.is_err());
    }
}

/// <https://learn.microsoft.com/en-us/windows/win32/multimedia/multimedia-command-strings>
fn send_mci(command: &str) -> Result<()> {
    debug!(command = %command, "Sending MCI command");
    let wide: Vec<u16> = OsStr::new(command)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // Safety: We provide a null-terminated buffer and pass null for optional pointers.
    let error = unsafe { mciSendStringW(PCWSTR(wide.as_ptr()), None, None) };
    if error != 0 {
        let message = mci_error_string(error)
            .unwrap_or_else(|| format!("MCI command '{command}' failed with code {error}"));
        debug!(code = error, message = %message, "MCI command failed");
        return Err(eyre::eyre!(message));
    }
    Ok(())
}

fn mci_error_string(code: u32) -> Option<String> {
    let mut buffer = vec![0u16; 512];
    let ok = unsafe { mciGetErrorStringW(code, &mut buffer).as_bool() };
    if !ok {
        return None;
    }
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..len]))
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn infer_types() {
        assert_eq!(infer_mci_type(&PathBuf::from("a.wav")), Some("waveaudio"));
        assert_eq!(infer_mci_type(&PathBuf::from("b.mp3")), Some("mpegvideo"));
        assert_eq!(infer_mci_type(&PathBuf::from("c.wma")), Some("mpegvideo"));
        assert_eq!(infer_mci_type(&PathBuf::from("d.unknown")), None);
    }
}
