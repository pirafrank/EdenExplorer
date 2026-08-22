# EdenExplorer performance benchmark run sheet

Run the release executable from PowerShell. Repeat each scenario with
`EDEN_RENDERER=wgpu`, `EDEN_RENDERER=glow`, and `EDEN_RENDERER=auto`.

## Environment record

Record these values once for each session:

```powershell
git rev-parse --short HEAD
$PSVersionTable.PSVersion
[Environment]::ProcessorCount
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM
Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber
```

Also record whether the session is the local console or RDP, screen
resolution, display scaling, and the directory used for the test.

## Launch

Use a separate PowerShell window for each run:

```powershell
$env:EDEN_RENDERER = "wgpu"
$env:EDEN_DIAGNOSTICS = "1"
& ".\target\release\EdenExplorer.exe" 2> ".\perf-wgpu.log"
```

Replace `wgpu` with `glow` and `auto`. Close the application after each run
and clear the environment variables before unrelated launches:

```powershell
Remove-Item Env:EDEN_RENDERER -ErrorAction SilentlyContinue
Remove-Item Env:EDEN_DIAGNOSTICS -ErrorAction SilentlyContinue
```

For the custom-chrome isolation run, additionally set
`EDEN_NATIVE_CHROME=1`. This enables native Windows decorations and bypasses
EdenExplorer's custom window procedure and DWM override. Repeat the move test
with the same renderer values, then unset the variable.

## Scenarios

Use the same directory and window size for every run.

1. **Idle:** leave the window stationary for 15 seconds.
2. **Window move:** continuously drag the custom title area for 10 seconds.
3. **Resize:** continuously resize from the lower-right corner for 10 seconds.
4. **Scroll:** scroll a directory containing many entries for 10 seconds with
   icons enabled.
5. **Navigation:** for approximately two minutes, open folders, go back and
   forward, switch tabs, select files, and open a context menu.

For each scenario record process CPU average and peak from Task Manager or
Process Explorer, plus responsiveness. For the window-move scenario, paste the
diagnostic lines from the log, including `ui_calls`, `ui_ms`,
`repaint_requests`, `rendered_frames`, `viewport_events`, and region timings.

## Result template

```text
Environment:
Session:
Renderer:
Commit:
CPU count:
Resolution / scaling:
Test directory:

Idle CPU avg/peak:
Move CPU avg/peak:
Resize CPU avg/peak:
Scroll CPU avg/peak:
Navigation CPU avg/peak:
Responsiveness:

Diagnostics during move:
```

Run the minimal renderer binary with the same renderer values:

```powershell
$env:EDEN_RENDERER = "wgpu"
& ".\target\release\renderer_bench.exe"
```

It intentionally uses native decorations and no EdenExplorer UI or window
procedure. A similar move cost in this binary points below EdenExplorer's
application UI; a large difference points toward custom chrome or application
repaint behavior.
