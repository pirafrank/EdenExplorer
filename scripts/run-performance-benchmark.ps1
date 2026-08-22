[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $Executable,

    [Parameter(Mandatory)]
    [ValidateSet("auto", "wgpu", "glow")]
    [string] $Renderer,

    [Parameter(Mandatory)]
    [string] $LogPath,

    [Parameter(Mandatory)]
    [ValidateSet("idle", "move", "resize", "scroll", "navigation")]
    [string] $Scenario
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not ("EdenPerformance.NativeMethods" -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

namespace EdenPerformance {
    public static class NativeMethods {
        public const uint INPUT_MOUSE = 0;
        public const uint INPUT_KEYBOARD = 1;
        public const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
        public const uint MOUSEEVENTF_LEFTUP = 0x0004;
        public const uint MOUSEEVENTF_WHEEL = 0x0800;
        public const uint KEYEVENTF_KEYUP = 0x0002;
        public const ushort VK_MENU = 0x12;
        public const ushort VK_LEFT = 0x25;
        public const ushort VK_RIGHT = 0x27;
        public const ushort VK_CONTROL = 0x11;
        public const ushort VK_SHIFT = 0x10;
        public const ushort VK_TAB = 0x09;
        public const ushort VK_ESCAPE = 0x1b;

        [StructLayout(LayoutKind.Sequential)]
        public struct RECT {
            public int Left;
            public int Top;
            public int Right;
            public int Bottom;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct MOUSEINPUT {
            public int dx;
            public int dy;
            public uint mouseData;
            public uint dwFlags;
            public uint time;
            public UIntPtr dwExtraInfo;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct KEYBDINPUT {
            public ushort wVk;
            public ushort wScan;
            public uint dwFlags;
            public uint time;
            public UIntPtr dwExtraInfo;
        }

        [StructLayout(LayoutKind.Explicit, Size = 40)]
        public struct INPUT {
            [FieldOffset(0)] public uint type;
            [FieldOffset(8)] public MOUSEINPUT mi;
            [FieldOffset(8)] public KEYBDINPUT ki;
        }

        [DllImport("user32.dll", SetLastError = true)]
        public static extern uint SendInput(uint count, INPUT[] inputs, int size);

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool SetForegroundWindow(IntPtr hwnd);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool ShowWindow(IntPtr hwnd, int command);

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool SetWindowPos(
            IntPtr hwnd,
            IntPtr insertAfter,
            int x,
            int y,
            int width,
            int height,
            uint flags);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool SetCursorPos(int x, int y);
    }
}
'@
}

function Send-MouseInput([uint32] $flags, [uint32] $data = 0) {
    $input = New-Object EdenPerformance.NativeMethods+INPUT
    $input.type = [EdenPerformance.NativeMethods]::INPUT_MOUSE
    $input.mi.dwFlags = $flags
    $input.mi.mouseData = $data
    $sent = [EdenPerformance.NativeMethods]::SendInput(
        1,
        @($input),
        [Runtime.InteropServices.Marshal]::SizeOf([type]"EdenPerformance.NativeMethods+INPUT"))
    if ($sent -eq 1) { $script:inputEventsSent++ }
}

function Send-Key([uint16] $key, [bool] $up = $false) {
    $input = New-Object EdenPerformance.NativeMethods+INPUT
    $input.type = [EdenPerformance.NativeMethods]::INPUT_KEYBOARD
    $input.ki.wVk = $key
    if ($up) {
        $input.ki.dwFlags = [EdenPerformance.NativeMethods]::KEYEVENTF_KEYUP
    }
    $sent = [EdenPerformance.NativeMethods]::SendInput(
        1,
        @($input),
        [Runtime.InteropServices.Marshal]::SizeOf([type]"EdenPerformance.NativeMethods+INPUT"))
    if ($sent -eq 1) { $script:inputEventsSent++ }
}

function Get-WindowRect([IntPtr] $handle) {
    $rect = New-Object EdenPerformance.NativeMethods+RECT
    if (-not [EdenPerformance.NativeMethods]::GetWindowRect($handle, [ref]$rect)) {
        throw "GetWindowRect failed for $handle"
    }
    return $rect
}

function Invoke-KeyChord([uint16[]] $keys) {
    foreach ($key in $keys) { Send-Key $key }
    foreach ($key in ($keys | Sort-Object -Descending)) { Send-Key $key $true }
}

function Invoke-ScenarioInput(
    [string] $name,
    [IntPtr] $handle,
    [EdenPerformance.NativeMethods+RECT] $initialRect,
    [Diagnostics.Stopwatch] $clock,
    [Diagnostics.Process] $process,
    [Collections.Generic.List[object]] $samples,
    [ref] $lastCpu,
    [ref] $lastSample
) {
    $centerX = [int](($initialRect.Left + $initialRect.Right) / 2)
    $centerY = [int](($initialRect.Top + $initialRect.Bottom) / 2)
    $topY = $initialRect.Top + 24
    $endX = $centerX + 240
    $endY = $topY + 120

    function Add-CpuSample {
        if ($clock.Elapsed - $lastSample.Value -lt [TimeSpan]::FromMilliseconds(250)) {
            return
        }
        $process.Refresh()
        $now = $clock.Elapsed
        $cpu = $process.TotalProcessorTime
        $wallDelta = ($now - $lastSample.Value).TotalSeconds
        $cpuDelta = ($cpu - $lastCpu.Value).TotalSeconds
        if ($wallDelta -gt 0) {
            $samples.Add([pscustomobject]@{
                elapsed_seconds = $now.TotalSeconds
                cpu_percent = ($cpuDelta / $wallDelta / [Environment]::ProcessorCount) * 100
            })
        }
        $lastCpu.Value = $cpu
        $lastSample.Value = $now
    }

    switch ($name) {
        "move" {
            [void][EdenPerformance.NativeMethods]::SetCursorPos($centerX, $topY)
            Send-MouseInput ([EdenPerformance.NativeMethods]::MOUSEEVENTF_LEFTDOWN)
            try {
                while ($clock.Elapsed.TotalSeconds -lt 10) {
                    $phase = [int]($clock.Elapsed.TotalMilliseconds / 500) % 2
                    if ($phase -eq 0) {
                        [void][EdenPerformance.NativeMethods]::SetCursorPos($endX, $endY)
                    } else {
                        [void][EdenPerformance.NativeMethods]::SetCursorPos($centerX, $topY)
                    }
                    Add-CpuSample
                    Start-Sleep -Milliseconds 25
                }
            } finally {
                Send-MouseInput ([EdenPerformance.NativeMethods]::MOUSEEVENTF_LEFTUP)
            }
        }
        "resize" {
            $startX = $initialRect.Right - 2
            $startY = $initialRect.Bottom - 2
            [void][EdenPerformance.NativeMethods]::SetCursorPos($startX, $startY)
            Send-MouseInput ([EdenPerformance.NativeMethods]::MOUSEEVENTF_LEFTDOWN)
            try {
                while ($clock.Elapsed.TotalSeconds -lt 10) {
                    $phase = [int]($clock.Elapsed.TotalMilliseconds / 500) % 2
                    if ($phase -eq 0) {
                        [void][EdenPerformance.NativeMethods]::SetCursorPos($startX + 160, $startY + 100)
                    } else {
                        [void][EdenPerformance.NativeMethods]::SetCursorPos($startX, $startY)
                    }
                    Add-CpuSample
                    Start-Sleep -Milliseconds 25
                }
            } finally {
                Send-MouseInput ([EdenPerformance.NativeMethods]::MOUSEEVENTF_LEFTUP)
            }
        }
        "scroll" {
            [void][EdenPerformance.NativeMethods]::SetCursorPos($centerX, $centerY)
            while ($clock.Elapsed.TotalSeconds -lt 10) {
                $wheel = if (([int]($clock.Elapsed.TotalMilliseconds / 500) % 2) -eq 0) { 120 } else { -120 }
                Send-MouseInput ([EdenPerformance.NativeMethods]::MOUSEEVENTF_WHEEL) ([uint32]$wheel)
                Add-CpuSample
                Start-Sleep -Milliseconds 100
            }
        }
        "navigation" {
            while ($clock.Elapsed.TotalSeconds -lt 120) {
                Invoke-KeyChord @(
                    [EdenPerformance.NativeMethods]::VK_MENU,
                    [EdenPerformance.NativeMethods]::VK_LEFT
                )
                Invoke-KeyChord @(
                    [EdenPerformance.NativeMethods]::VK_MENU,
                    [EdenPerformance.NativeMethods]::VK_RIGHT
                )
                Invoke-KeyChord @(
                    [EdenPerformance.NativeMethods]::VK_CONTROL,
                    [EdenPerformance.NativeMethods]::VK_TAB
                )
                Invoke-KeyChord @(
                    [EdenPerformance.NativeMethods]::VK_CONTROL,
                    [EdenPerformance.NativeMethods]::VK_SHIFT,
                    [EdenPerformance.NativeMethods]::VK_TAB
                )
                Send-Key [EdenPerformance.NativeMethods]::VK_ESCAPE
                Send-Key [EdenPerformance.NativeMethods]::VK_ESCAPE $true
                Add-CpuSample
                Start-Sleep -Milliseconds 500
            }
        }
        "idle" {
            while ($clock.Elapsed.TotalSeconds -lt 15) {
                Add-CpuSample
                Start-Sleep -Milliseconds 100
            }
        }
    }
}

$resolvedExecutable = (Resolve-Path $Executable).Path
$resolvedLog = [IO.Path]::GetFullPath($LogPath)
$resolvedDirectory = Split-Path -Parent $resolvedLog
New-Item -ItemType Directory -Force -Path $resolvedDirectory | Out-Null

$oldRenderer = $env:EDEN_RENDERER
$oldDiagnostics = $env:EDEN_DIAGNOSTICS
$oldDiagnosticsFile = $env:EDEN_DIAGNOSTICS_FILE
$process = $null
$samples = [Collections.Generic.List[object]]::new()
$script:inputEventsSent = 0

try {
    $env:EDEN_RENDERER = $Renderer
    $env:EDEN_DIAGNOSTICS = "1"
    $env:EDEN_DIAGNOSTICS_FILE = $resolvedLog

    $process = Start-Process `
        -FilePath $resolvedExecutable `
        -WorkingDirectory (Split-Path $resolvedExecutable) `
        -Environment @{
            EDEN_RENDERER = $Renderer
            EDEN_DIAGNOSTICS = "1"
            EDEN_DIAGNOSTICS_FILE = $resolvedLog
        } `
        -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    do {
        Start-Sleep -Milliseconds 100
        $process.Refresh()
        if ($process.HasExited) {
            throw "$resolvedExecutable exited before creating a window (exit code $($process.ExitCode))"
        }
    } while ($process.MainWindowHandle -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $deadline)

    if ($process.MainWindowHandle -eq [IntPtr]::Zero) {
        throw "Timed out waiting for the main window"
    }

    [void][EdenPerformance.NativeMethods]::SetForegroundWindow($process.MainWindowHandle)
    [void][EdenPerformance.NativeMethods]::ShowWindow($process.MainWindowHandle, 5)
    $sizeDeadline = [DateTime]::UtcNow.AddSeconds(20)
    do {
        $initialRect = Get-WindowRect $process.MainWindowHandle
        $width = $initialRect.Right - $initialRect.Left
        $height = $initialRect.Bottom - $initialRect.Top
        if ($width -gt 100 -and $height -gt 100) {
            break
        }
        Start-Sleep -Milliseconds 100
    } while (-not $process.HasExited -and [DateTime]::UtcNow -lt $sizeDeadline)
    if ($width -le 100 -or $height -le 100) {
        [void][EdenPerformance.NativeMethods]::SetWindowPos(
            $process.MainWindowHandle,
            [IntPtr]::Zero,
            100,
            100,
            800,
            600,
            0x0040)
        $initialRect = Get-WindowRect $process.MainWindowHandle
        $width = $initialRect.Right - $initialRect.Left
        $height = $initialRect.Bottom - $initialRect.Top
    }
    if ($width -le 100 -or $height -le 100) {
        throw "Window did not reach a measurable size after SetWindowPos"
    }
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $lastCpu = $process.TotalProcessorTime
    $lastSample = [TimeSpan]::Zero
    Invoke-ScenarioInput $Scenario $process.MainWindowHandle $initialRect $clock $process $samples ([ref]$lastCpu) ([ref]$lastSample)
    $clock.Stop()
    $process.Refresh()
    $finalRect = Get-WindowRect $process.MainWindowHandle

    $wallSeconds = [Math]::Max($clock.Elapsed.TotalSeconds, 0.001)
    $cpuSeconds = ($process.TotalProcessorTime - $lastCpu).TotalSeconds
    $averageCpu = ($cpuSeconds / $wallSeconds / [Environment]::ProcessorCount) * 100
    $peakCpu = if ($samples.Count -gt 0) {
        ($samples | Measure-Object -Property cpu_percent -Maximum).Maximum
    } else {
        $averageCpu
    }
    $inputSent = $inputEventsSent -gt 0
    $rectChanged = $initialRect.Left -ne $finalRect.Left -or
        $initialRect.Top -ne $finalRect.Top -or
        $initialRect.Right -ne $finalRect.Right -or
        $initialRect.Bottom -ne $finalRect.Bottom

    $result = [ordered]@{
        executable = $resolvedExecutable
        renderer = $Renderer
        scenario = $Scenario
        pid = $process.Id
        elapsed_seconds = [Math]::Round($wallSeconds, 3)
        average_cpu_percent = [Math]::Round($averageCpu, 2)
        peak_cpu_percent = [Math]::Round($peakCpu, 2)
        sample_count = $samples.Count
        input_sent = $inputSent
        window_rect_changed = $rectChanged
        initial_rect = $initialRect
        final_rect = $finalRect
        timestamp_utc = [DateTime]::UtcNow.ToString("o")
    }
    $jsonPath = [IO.Path]::ChangeExtension($resolvedLog, ".json")
    $result | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 $jsonPath
    $result | ConvertTo-Json -Compress
} finally {
    if ($null -ne $process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
    if ($null -eq $oldRenderer) { Remove-Item Env:EDEN_RENDERER -ErrorAction SilentlyContinue } else { $env:EDEN_RENDERER = $oldRenderer }
    if ($null -eq $oldDiagnostics) { Remove-Item Env:EDEN_DIAGNOSTICS -ErrorAction SilentlyContinue } else { $env:EDEN_DIAGNOSTICS = $oldDiagnostics }
    if ($null -eq $oldDiagnosticsFile) { Remove-Item Env:EDEN_DIAGNOSTICS_FILE -ErrorAction SilentlyContinue } else { $env:EDEN_DIAGNOSTICS_FILE = $oldDiagnosticsFile }
}
