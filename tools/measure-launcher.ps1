<#
.SYNOPSIS
  Measures what a launcher prototype costs on Windows: start time, memory,
  CPU and GPU, at rest in the menu and while a game runs.
  Mesure ce que coûte un prototype de lanceur sous Windows : démarrage,
  mémoire, processeur et carte graphique, au repos et pendant un jeu.

.DESCRIPTION
  The launcher and every child process are counted together (WebView2 runs
  in several msedgewebview2.exe processes, and the Qt prototype keeps a
  clctl.exe process waiting during a game). The emulator and the game are
  NOT counted: only the launcher's own cost is measured.
  Le lanceur et tous ses processus enfants sont comptés ensemble. L'émulateur
  et le jeu ne sont PAS comptés : seul le coût propre du lanceur est mesuré.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\console-launcher.exe -Name tauri
  powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\console-launcher-qt.exe -Name qt
  powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\ConsoleLauncher.exe -Name wpf
#>
param(
    [Parameter(Mandatory = $true)] [string] $Exe,
    [Parameter(Mandatory = $true)] [string] $Name,
    [int] $Seconds = 30,
    [string[]] $IgnoreNames = @("Dolphin", "pcsx2-qt", "retroarch", "PresentMon", "MangoHud")
)

$ErrorActionPreference = "Stop"
$cores = [Environment]::ProcessorCount

function Get-Tree([int] $rootPid) {
    $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
    $ids = New-Object System.Collections.Generic.HashSet[int]
    [void] $ids.Add($rootPid)
    do {
        $added = $false
        foreach ($p in $all) {
            if ($ids.Contains([int] $p.ParentProcessId) -and -not $ids.Contains([int] $p.ProcessId)) {
                $base = [IO.Path]::GetFileNameWithoutExtension($p.Name)
                if ($IgnoreNames -notcontains $base) { [void] $ids.Add([int] $p.ProcessId); $added = $true }
            }
        }
    } while ($added)
    return $ids
}

function Get-GpuPercent($ids) {
    try {
        $samples = (Get-Counter '\GPU Engine(*)\Utilization Percentage' -ErrorAction Stop).CounterSamples
        $sum = 0.0
        foreach ($s in $samples) {
            if ($s.InstanceName -match 'pid_(\d+)_' -and $ids.Contains([int] $Matches[1])) { $sum += $s.CookedValue }
        }
        return [math]::Round($sum, 2)
    } catch { return $null }
}

function Measure-Phase([int] $rootPid, [string] $phase) {
    Write-Host "  $phase : $Seconds s…"
    $ids = Get-Tree $rootPid
    $procs = @(Get-Process -Id ([int[]] $ids) -ErrorAction SilentlyContinue)
    $cpu0 = ($procs | ForEach-Object { $_.TotalProcessorTime.TotalMilliseconds } | Measure-Object -Sum).Sum
    $gpu = @(); $mem = @()
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $Seconds) {
        Start-Sleep -Seconds 2
        $ids = Get-Tree $rootPid
        $procs = @(Get-Process -Id ([int[]] $ids) -ErrorAction SilentlyContinue)
        $mem += (($procs | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB)
        $g = Get-GpuPercent $ids
        if ($g -ne $null) { $gpu += $g }
    }
    $procs = @(Get-Process -Id ([int[]] $ids) -ErrorAction SilentlyContinue)
    $cpu1 = ($procs | ForEach-Object { $_.TotalProcessorTime.TotalMilliseconds } | Measure-Object -Sum).Sum
    $elapsed = $sw.Elapsed.TotalMilliseconds
    [pscustomobject]@{
        prototype      = $Name
        phase          = $phase
        processes      = $procs.Count
        memory_mb      = [math]::Round(($mem | Measure-Object -Average).Average, 1)
        memory_max_mb  = [math]::Round(($mem | Measure-Object -Maximum).Maximum, 1)
        cpu_percent    = [math]::Round(100 * ($cpu1 - $cpu0) / $elapsed / $cores, 2)
        gpu_percent    = if ($gpu.Count) { [math]::Round(($gpu | Measure-Object -Average).Average, 2) } else { $null }
    }
}

$exePath = (Resolve-Path $Exe).Path
Write-Host "== $Name : $exePath"
$sw = [Diagnostics.Stopwatch]::StartNew()
$proc = Start-Process -FilePath $exePath -WorkingDirectory (Split-Path $exePath) -PassThru
while ($proc.MainWindowHandle -eq 0 -and $sw.Elapsed.TotalSeconds -lt 30) { Start-Sleep -Milliseconds 20; $proc.Refresh() }
$startMs = [math]::Round($sw.Elapsed.TotalMilliseconds)
Write-Host "  window after $startMs ms. Let it settle 5 s, do not touch anything / ne touchez à rien."
Start-Sleep -Seconds 5

$results = @()
$results += Measure-Phase $proc.Id "menu (idle / repos)"
Write-Host ""
Write-Host "  Now start a game from the launcher, then press Enter here once it is playing."
Write-Host "  Lancez maintenant un jeu depuis le lanceur, puis appuyez sur Entrée ici quand il tourne."
[void] (Read-Host)
$results += Measure-Phase $proc.Id "in game (launcher only / lanceur seul)"

foreach ($r in $results) { $r | Add-Member startup_ms $startMs }
$results | Format-Table -AutoSize
$out = Join-Path (Get-Location) "measure-$Name.csv"
$results | Export-Csv -NoTypeInformation -Encoding UTF8 $out
Write-Host "Saved / enregistré : $out"
