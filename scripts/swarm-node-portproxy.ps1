<#
.SYNOPSIS
    Keeps a Windows netsh portproxy rule pointed at the current WSL2
    internal IP, so a swarm-node listening inside WSL2 with --bind 0.0.0.0
    stays reachable from another machine (e.g. over Tailscale) even after
    a WSL restart changes the VM's internal address.

.DESCRIPTION
    See docs/swarm-mesh-v2.md, M0.11: Windows only auto-forwards
    127.0.0.1 into a WSL2 VM by default, not other host interfaces (like
    a Tailscale adapter). A one-time `netsh interface portproxy` rule
    bridges <listen-address>:<port> -> <wsl2-internal-ip>:<port>, but the
    WSL2-internal IP is not stable across WSL restarts, which silently
    breaks external reachability with no error anywhere -- my-lisp-1
    would keep working fine locally while looking dead to remote peers.

    This script re-detects the current WSL2 internal IP and re-applies
    the portproxy rule idempotently (removes any stale rule for the same
    listen address/port first, so re-running is always safe). It does
    NOT start swarm-node itself, decide the listen address, or set up
    scheduling -- it only fixes the network path. Run it after every WSL
    restart, or wire it into whatever starts swarm-node.

.PARAMETER ListenAddress
    The Windows-side address remote peers connect to (typically this
    host's Tailscale IPv4 address). Required.

.PARAMETER Port
    The swarm-node port. Defaults to 9101 (the my-lisp-1 bootstrap port
    used throughout docs/swarm-mesh-v2.md).

.PARAMETER WslDistro
    The WSL distro name to query for its internal IP. Defaults to the
    default distro (omit -d entirely) if not specified.

.EXAMPLE
    # Elevated PowerShell:
    .\scripts\swarm-node-portproxy.ps1 -ListenAddress 100.120.29.6

.EXAMPLE
    .\scripts\swarm-node-portproxy.ps1 -ListenAddress 100.120.29.6 -Port 9105 -WslDistro Ubuntu
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$ListenAddress,

    [int]$Port = 9101,

    [string]$WslDistro
)

$ErrorActionPreference = 'Stop'

$currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must run in an elevated (Administrator) PowerShell -- netsh interface portproxy requires it."
    exit 1
}

if ($WslDistro) {
    $wslIpRaw = & wsl.exe -d $WslDistro -- hostname -I
} else {
    $wslIpRaw = & wsl.exe -- hostname -I
}
$wslIp = ($wslIpRaw -split '\s+')[0].Trim()
if ([string]::IsNullOrWhiteSpace($wslIp)) {
    Write-Error "Could not determine the WSL2 internal IP (got: '$wslIpRaw'). Is WSL running?"
    exit 1
}

Write-Host "WSL2 internal IP: $wslIp"
Write-Host "Mapping ${ListenAddress}:${Port} -> ${wslIp}:${Port}"

# Idempotent: delete any existing rule for this listen address/port first
# (netsh add fails instead of updating if one already exists), then add
# the current mapping fresh.
& netsh interface portproxy delete v4tov4 listenaddress=$ListenAddress listenport=$Port 2>$null | Out-Null
& netsh interface portproxy add v4tov4 listenaddress=$ListenAddress listenport=$Port connectaddress=$wslIp connectport=$Port

Write-Host "Current portproxy rules:"
& netsh interface portproxy show v4tov4
