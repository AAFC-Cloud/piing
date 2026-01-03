cargo build --release
$old_exe = Get-Command piing.exe | Select-Object -ExpandProperty Source
if (-not (Test-Path $old_exe)) {
    Write-Error "Could not find piing.exe in your path!"
    return
}
$new_exe = "target\release\piing.exe"
if (-not (Test-Path $new_exe)) {
    Write-Error "Could not find target exe, run `cargo build --release` please."
    return
}

# If piing is running, stop it before replacing the executable
$running = Get-Process -Name piing -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "piing.exe is currently running (pid(s): $($running.Id -join ', ')). Stopping..."
    # Try to stop gracefully, then force if needed
    try {
        $running | Stop-Process -ErrorAction Stop
    } catch {
        Write-Warning "Could not stop some piing processes gracefully, forcing stop."
        $running | Stop-Process -Force -ErrorAction SilentlyContinue
    }

    # Wait up to 10s for processes to exit
    $wait = 0
    while ((Get-Process -Name piing -ErrorAction SilentlyContinue) -and $wait -lt 10) {
        Start-Sleep -Seconds 1
        $wait++
    }

    if (Get-Process -Name piing -ErrorAction SilentlyContinue) {
        Write-Error "Could not stop piing.exe after waiting; aborting update to avoid a locked file."
        return
    }
}

# Replace the executable in PATH with the newly built one
Copy-Item -Path $new_exe -Destination $old_exe -Force

# Start the new copy of piing
try {
    Start-Process -FilePath $old_exe -ErrorAction Stop
    Write-Host "piing.exe restarted."
} catch {
    Write-Warning "Failed to restart piing.exe automatically. You may need to start it manually."
}
Write-Host "Now in path:"
piing --version