# Requires -RunAsAdministrator
# Self-elevating PowerShell script to toggle a network adapter off for 3 seconds, then back on.

# --- Configuration ---
$adapter = "Ethernet 2"

function Ensure-Elevated {
  $currentIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = New-Object Security.Principal.WindowsPrincipal($currentIdentity)
  $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)

  if (-not $isAdmin) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = (Get-Process -Id $PID).Path
    $psi.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    $psi.Verb = "runas"
    try {
      $p = [System.Diagnostics.Process]::Start($psi)
      if ($p) { exit }
    } catch {
      Write-Error "Elevation was canceled or failed. Exiting."
      exit 1
    }
  }
}

function Main {
  try {
    Write-Host "Disabling adapter '$adapter'..." -ForegroundColor Yellow
    Disable-NetAdapter -Name $adapter -Confirm:$false -ErrorAction Stop

    Write-Host "Sleeping for 3 seconds..." -ForegroundColor Cyan
    Start-Sleep -Seconds 3
  } catch {
    Write-Error "Error while disabling adapter: $($_.Exception.Message)"
  } finally {
    try {
      Write-Host "Re-enabling adapter '$adapter'..." -ForegroundColor Yellow
      Enable-NetAdapter -Name $adapter -ErrorAction Stop
      Write-Host "Adapter re-enabled." -ForegroundColor Green
    } catch {
      Write-Error "Failed to re-enable adapter: $($_.Exception.Message)"
      Write-Host "You may need to re-enable it manually: Enable-NetAdapter -Name `"$adapter`""
    }
  }
}

Ensure-Elevated
Main