param(
    [string]$DaemonPath,
    [string]$DesktopPath,
    [switch]$PrepareOnly
)
$ErrorActionPreference = 'Stop'
if (-not $IsWindows -or $PSVersionTable.PSVersion -lt [version]'7.4') {
    throw 'Use PowerShell 7.4 or newer on Windows.'
}
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if (-not $DaemonPath) { $DaemonPath = Join-Path $repository 'target/release/sd-daemon.exe' }
if (-not $DesktopPath) { $DesktopPath = Join-Path $repository 'target/debug/Spacedrive.exe' }
$DaemonPath = [IO.Path]::GetFullPath($DaemonPath)
$DesktopPath = [IO.Path]::GetFullPath($DesktopPath)
if (-not $PrepareOnly) {
    foreach ($binary in @($DaemonPath, $DesktopPath)) {
        if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw "Build the executable first: $binary" }
    }
}

$instance = 'workspace-' + [Guid]::NewGuid().ToString('N').Substring(0, 12)
# This is the named-instance address contract in core/src/infra/daemon/addr.rs.
$byteSum = 0
foreach ($value in [Text.Encoding]::UTF8.GetBytes($instance)) { $byteSum += $value }
$port = 6970 + ($byteSum % 1000)
$testRoot = Join-Path $repository ('.workspace-test/' + [Guid]::NewGuid().ToString('N'))
$dataDirectory = Join-Path $testRoot ('instances/' + $instance)
New-Item -ItemType Directory -Path $dataDirectory | Out-Null
$config = [ordered]@{
    version = 6
    data_dir = $dataDirectory
    log_level = 'info'
    telemetry_enabled = $false
    preferences = @{ theme = 'dark'; language = 'en' }
    services = @{
        networking_enabled = $false
        volume_monitoring_enabled = $false
        fs_watcher_enabled = $true
        statistics_listener_enabled = $false
    }
}
$config | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $dataDirectory 'spacedrive.json') -Encoding utf8
$manifest = [ordered]@{
    instance = $instance
    daemonAddress = "127.0.0.1:$port"
    dataDirectory = $dataDirectory
    webviewDirectory = Join-Path $dataDirectory 'webview'
    daemonPath = $DaemonPath
    desktopPath = $DesktopPath
    status = 'prepared'
}
$manifestPath = Join-Path $testRoot 'launch.json'
function Save-Manifest { $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8 }
Save-Manifest
Write-Output "Test manifest: $manifestPath"
if ($PrepareOnly) { return }

function Test-DaemonPing {
    $client = [Net.Sockets.TcpClient]::new()
    try {
        if (-not $client.ConnectAsync('127.0.0.1', $port).Wait(300)) { return $false }
        $stream = $client.GetStream()
        $stream.ReadTimeout = 500
        $writer = [IO.StreamWriter]::new($stream)
        $writer.AutoFlush = $true
        $writer.WriteLine('{"Ping":null}')
        $reader = [IO.StreamReader]::new($stream)
        return ($reader.ReadLine() -eq '"Pong"')
    } catch { return $false } finally { $client.Dispose() }
}

$daemon = $null
$desktop = $null
try {
    $portProbe = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, $port)
    try { $portProbe.Start() } finally { $portProbe.Stop() }
    $daemon = Start-Process -FilePath $DaemonPath -WorkingDirectory (Split-Path $DaemonPath) -ArgumentList @('--data-dir', ('"' + $testRoot + '"'), '--instance', $instance, '--no-default-sources') -Environment @{RUST_LOG='info';SD_MOUNT_HTTP_PORT='0'} -WindowStyle Hidden -RedirectStandardOutput (Join-Path $testRoot 'daemon.stdout.log') -RedirectStandardError (Join-Path $testRoot 'daemon.stderr.log') -PassThru
    $manifest.daemonPid = $daemon.Id
    Save-Manifest
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-DaemonPing)) {
        if ($daemon.HasExited) { throw "Daemon exited: $($daemon.ExitCode). See $testRoot" }
        if ([DateTime]::UtcNow -ge $deadline) { throw "Daemon did not become ready. See $testRoot" }
        Start-Sleep -Milliseconds 200
    }
    $listeners = @(Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction Stop)
    if (-not ($listeners | Where-Object OwningProcess -eq $daemon.Id)) { throw 'The listening daemon is not the process started by this test.' }
    $manifest.daemonSha256 = (Get-FileHash -LiteralPath $DaemonPath -Algorithm SHA256).Hash
    $manifest.desktopSha256 = (Get-FileHash -LiteralPath $DesktopPath -Algorithm SHA256).Hash
    $desktop = Start-Process -FilePath $DesktopPath -WorkingDirectory (Split-Path $DesktopPath) -Environment @{SD_WORKSPACE_TEST_DATA_DIR=$dataDirectory;SD_WORKSPACE_TEST_DAEMON_ADDR="127.0.0.1:$port";RUST_LOG='info'} -WindowStyle Hidden -RedirectStandardOutput (Join-Path $testRoot 'desktop.stdout.log') -RedirectStandardError (Join-Path $testRoot 'desktop.stderr.log') -PassThru
    $manifest.desktopPid = $desktop.Id
    $manifest.status = 'running'
    Save-Manifest
    $desktop.WaitForExit()
    $manifest.status = 'desktop-exited'
} catch {
    $manifest.status = 'failed'
    $manifest.error = $_.Exception.Message
    throw
} finally {
    if ($desktop -and -not $desktop.HasExited) { $desktop.Kill() }
    if ($daemon -and -not $daemon.HasExited) { $daemon.Kill() }
    Save-Manifest
}
