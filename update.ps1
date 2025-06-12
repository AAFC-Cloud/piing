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
Copy-Item -Path $new_exe -Destination $old_exe
Write-Host "Now in path:"
piing --version