# 8h 长稳 / CPU 采样脚本（UI-15、AUD-11、REL 门禁证据）
#
# 用法（应用已在播放队列时运行）：
#   powershell -ExecutionPolicy Bypass -File scripts\soak.ps1 -Hours 8
#   powershell -ExecutionPolicy Bypass -File scripts\soak.ps1 -Hours 0.05 -IntervalSeconds 2 -OutFile .tmp-build/soak.csv
#
# 输出：CSV（时间、工作集 MB、私有内存 MB、CPU 秒、线程数、句柄数、是否响应），
# 结束后打印峰值/均值。CPU 使用率 = 两次采样的 CPU 秒差 / 采样间隔。

param(
    [double]$Hours = 8,
    [int]$IntervalSeconds = 60,
    [string]$ProcessName = "BiuPlayer",
    [string]$OutFile = "soak.csv"
)

$deadline = (Get-Date).AddHours($Hours)
$rows = @()
$prevCpu = $null
$prevTime = $null
$missing = 0

"time,working_set_mb,private_mb,cpu_percent,threads,handles,responding" | Set-Content -Path $OutFile -Encoding utf8

while ((Get-Date) -lt $deadline) {
    $proc = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $proc) {
        $missing++
        if ($missing -ge 3) {
            Write-Warning "process '$ProcessName' not running; stopping soak"
            break
        }
        Start-Sleep -Seconds $IntervalSeconds
        continue
    }
    $missing = 0

    $now = Get-Date
    $cpu = $proc.CPU
    $cpuPercent = 0.0
    if ($null -ne $prevCpu -and $null -ne $prevTime) {
        $elapsed = ($now - $prevTime).TotalSeconds
        if ($elapsed -gt 0) {
            $cpuPercent = [math]::Round((($cpu - $prevCpu) / $elapsed) * 100, 1)
        }
    }
    $prevCpu = $cpu
    $prevTime = $now

    $ws = [math]::Round($proc.WorkingSet64 / 1MB, 1)
    $priv = [math]::Round($proc.PrivateMemorySize64 / 1MB, 1)
    $responding = -not $proc.HasExited -and $proc.Responding

    $line = "{0:o},{1},{2},{3},{4},{5},{6}" -f $now, $ws, $priv, $cpuPercent, $proc.Threads.Count, $proc.HandleCount, $responding
    $line | Add-Content -Path $OutFile -Encoding utf8
    $rows += [pscustomobject]@{ WorkingSet = $ws; Private = $priv; Cpu = $cpuPercent; Responding = $responding }

    Start-Sleep -Seconds $IntervalSeconds
}

if ($rows.Count -gt 0) {
    $avgCpu = [math]::Round(($rows | Measure-Object -Property Cpu -Average).Average, 1)
    $maxCpu = [math]::Round(($rows | Measure-Object -Property Cpu -Maximum).Maximum, 1)
    $maxWs = [math]::Round(($rows | Measure-Object -Property WorkingSet -Maximum).Maximum, 1)
    $unresponsive = ($rows | Where-Object { -not $_.Responding }).Count
    Write-Host ("samples={0} avg_cpu={1}% max_cpu={2}% max_working_set={3}MB unresponsive={4}" -f $rows.Count, $avgCpu, $maxCpu, $maxWs, $unresponsive)
    Write-Host "csv: $OutFile"
} else {
    Write-Warning "no samples collected"
}
