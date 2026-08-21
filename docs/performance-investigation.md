# EdenExplorer Windows rendering investigation

Status: instrumentation and adaptive backend implementation are in place. The
machine matrix below remains to be filled from the Proxmox VM and GPU
workstation runs; this checkout does not have access to those environments.

## Reproduction protocol

Record the commit, Rust/Cargo versions, profile, executable size, dependency
feature tree, Windows build, CPU count, GPU/display adapter, resolution, DPI,
and whether the process is in a local console or RDP session. Use the same
directory for every run, with 2,000 and 10,000 entries where available.

For each renderer (`EDEN_RENDERER=wgpu` and `EDEN_RENDERER=glow`) run five
minutes of idle, pointer hover, window movement, resize, scroll, ordinary
navigation, icon/thumbnail completion, clipboard update, and device update.
Capture average and peak process CPU, responsiveness, frame rate, repaint
rate, and the diagnostic counters. Repeat on VM console, VM RDP, GPU console,
and GPU RDP when available. Set `EDEN_DIAGNOSTICS=1` for the diagnostic run.

The relevant commands are:

```text
cargo tree -e features
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo build --release
set EDEN_RENDERER=auto|wgpu|glow
set EDEN_DIAGNOSTICS=1
```

The diagnostic line reports `ui_calls`, accumulated `ui_ms`, explicit repaint
requests, application frame callbacks, and selected window events. The frame
counter is an application callback counter, not a claim that the backend
presented a new image; eframe 0.35 does not expose a general present callback
through `App`.

## Implementation findings

- eframe 0.35 exposes explicit native `wgpu` and `glow` renderer selection.
  `auto` currently selects wgpu and logs whether the process is local or RDP.
  RDP is reported but is not used as a performance proxy.
- An explicit `wgpu` initialization failure in `auto` retries once with glow.
  Explicit overrides do not silently change backend.
- Invalid `EDEN_RENDERER` values produce a clear diagnostic and use `auto`.
- The app uses borderless custom chrome. `windowsoverrides.rs` installs a
  custom `WndProc`, applies DWM attributes, and handles non-client behavior.
  Its `WM_MOVE`, `WM_SIZE`, `WM_WINDOWPOSCHANGED`, `WM_PAINT`, and
  `WM_EXITSIZEMOVE` traffic is now countable for comparison with a native
  decorations build.
- The source already virtualizes gallery rows and egui performs clip-aware
  tessellation. No retained framebuffer or application damage-region API is
  present in eframe's native `App` path.

## Partial redraw conclusion

Clip rectangles reduce primitive rasterization, but do not by themselves skip
immediate-mode UI evaluation, buffer submission, or presentation. The safe
optimization target is repaint scheduling and UI construction cost. A custom
region compositor is not justified by source inspection and must not be added
unless the cross-environment measurements show a material, backend-supported
benefit without breaking interaction, scrolling, themes, accessibility, or
custom chrome.

The next evidence to collect is whether movement causes application callbacks
and repaint requests, or whether CPU time is concentrated below `App::ui()` in
the renderer/compositor. If `ui_ms` remains low while CPU is saturated, use
WPR/WPA/ETW with CPU sampling and GPU/DWM providers; application counters cannot
classify that cost.

## Feature-tree decision

The eframe default feature set was replaced with `accesskit`, `default_fonts`,
`glow`, and `wgpu`. Wayland, X11, web-screen-reader, and Android paths are no
longer enabled by this application. `egui/serde` remains enabled because
serialized theme values contain egui types. `egui_extras` and `rfd` no longer
enable their non-Windows defaults. Image defaults remain until thumbnail format
coverage is explicitly inventoried; narrowing them prematurely could remove
supported file formats. eframe 0.35's native `egui-winit` dependency still
enables built-in clipboard and hyperlink (`links`) support; those are not
independently removable from the application dependency declaration.

## Results table

| Environment | Renderer | Scenario | CPU avg/peak | UI calls/ms | Repaints/frames | Result |
|---|---|---|---:|---:|---:|---|
| pending | wgpu/glow | idle/move/resize/scroll | pending | pending | pending | pending |
