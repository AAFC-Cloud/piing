use crate::config::ProblemSound;
use eyre::Result;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use tracing::warn;
use windows::Win32::Media::Multimedia::mciGetErrorStringW;
use windows::Win32::Media::Multimedia::mciSendStringW;
use windows::core::PCWSTR;

static ALIAS_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Play the configured problem sound on a background thread.
///
/// # Errors
/// Returns an error if the sound cannot be scheduled for playback.
pub fn play_problem_sound(sound: &ProblemSound) -> Result<()> {
    let path = sound.path().to_path_buf();
    let volume = sound.volume();
    thread::spawn(move || {
        if let Err(error) = play_sound_internal(&path, volume) {
            warn!("Failed to play problem sound: {error}");
        }
    });
    Ok(())
}

fn play_sound_internal(path: &Path, volume: f32) -> Result<()> {
    if !path.exists() {
        return Err(eyre::eyre!(
            "Problem sound path does not exist: {}",
            path.display()
        ));
    }

    let alias_id = ALIAS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let alias = format!("piing_problem_sound_{alias_id}");

    let open_cmd = format!(
        "open \"{}\" type waveaudio alias {alias}",
        path.to_string_lossy()
    );
    send_mci(&open_cmd)?;

    let result = (|| {
        let clamped = volume.clamp(0.0, 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let volume_value = (clamped * 1000.0).round() as u32;
        let set_cmd = format!("setaudio {alias} volume to {volume_value}");
        send_mci(&set_cmd)?;

        let play_cmd = format!("play {alias} from 0 wait");
        send_mci(&play_cmd)
    })();

    let close_cmd = format!("close {alias}");
    let _ = send_mci(&close_cmd);

    result
}

fn send_mci(command: &str) -> Result<()> {
    let wide: Vec<u16> = OsStr::new(command)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // Safety: We provide a null-terminated buffer and pass null for optional pointers.
    let error = unsafe { mciSendStringW(PCWSTR(wide.as_ptr()), None, None) };
    if error != 0 {
        let message = mci_error_string(error)
            .unwrap_or_else(|| format!("MCI command '{command}' failed with code {error}"));
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
