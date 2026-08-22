//! Windows rendering policy and low-overhead diagnostics.
//!
//! The diagnostics are intentionally environment-gated. Set
//! `EDEN_DIAGNOSTICS=1` when comparing renderers or investigating repaint
//! behavior; normal users pay only for a few disabled atomic operations.

use eframe::Renderer;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RendererRequest {
    Auto,
    Wgpu,
    Glow,
}

#[derive(Clone, Copy, Debug)]
pub struct RendererSelection {
    pub request: RendererRequest,
    pub selected: Renderer,
}

impl RendererSelection {
    pub fn renderer(self) -> Renderer {
        self.selected
    }

    pub fn can_fallback(self) -> bool {
        self.request == RendererRequest::Auto && self.selected == Renderer::Wgpu
    }
}

pub fn select_renderer() -> RendererSelection {
    let remote_session = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_REMOTESESSION,
        ) != 0
    };
    log_wgpu_adapters();
    let raw = std::env::var("EDEN_RENDERER").unwrap_or_else(|_| "auto".to_owned());
    let request = match raw.trim().to_ascii_lowercase().as_str() {
        "auto" | "" => RendererRequest::Auto,
        "wgpu" => RendererRequest::Wgpu,
        "glow" => RendererRequest::Glow,
        invalid => {
            report_line(format!(
                "EdenExplorer: invalid EDEN_RENDERER={invalid:?}; using auto (expected auto, wgpu, or glow)"
            ));
            RendererRequest::Auto
        }
    };

    // Auto deliberately starts with wgpu. RDP status is reported for
    // comparison, but is not treated as a proxy for renderer performance.
    let selected = match request {
        RendererRequest::Auto | RendererRequest::Wgpu => Renderer::Wgpu,
        RendererRequest::Glow => Renderer::Glow,
    };
    let selection = RendererSelection { request, selected };
    report_line(format!(
        "EdenExplorer rendering: requested={:?}, selected={}, session={}, compiled_backends=wgpu,glow",
        selection.request,
        selection.selected,
        if remote_session {
            "RDP"
        } else {
            "console/local"
        },
    ));
    selection
}

pub fn native_chrome_requested() -> bool {
    std::env::var("EDEN_NATIVE_CHROME")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}

fn log_wgpu_adapters() {
    if !diagnostics_requested() {
        return;
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    if adapters.is_empty() {
        report_line("EdenExplorer diagnostics: wgpu adapters=none".to_owned());
        return;
    }

    for adapter in adapters {
        let info = adapter.get_info();
        report_line(format!(
            "EdenExplorer diagnostics wgpu adapter: name={:?} backend={:?} device_type={:?} driver={:?} driver_info={:?} vendor=0x{:04x} device=0x{:04x}",
            info.name,
            info.backend,
            info.device_type,
            info.driver,
            info.driver_info,
            info.vendor,
            info.device,
        ));
    }
}

fn diagnostics_requested() -> bool {
    std::env::var("EDEN_DIAGNOSTICS")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}

static DIAGNOSTICS: AtomicU64 = AtomicU64::new(0);
static UI_CALLS: AtomicU64 = AtomicU64::new(0);
static UI_NANOS: AtomicU64 = AtomicU64::new(0);
static REPAINT_REQUESTS: AtomicU64 = AtomicU64::new(0);
static RENDERED_FRAMES: AtomicU64 = AtomicU64::new(0);
static VIEWPORT_EVENTS: AtomicU64 = AtomicU64::new(0);
static LAST_REPORT: Mutex<Option<Instant>> = Mutex::new(None);
static REGION_TIMINGS: OnceLock<Mutex<BTreeMap<&'static str, (u64, u64)>>> = OnceLock::new();
static DIAGNOSTIC_OUTPUT: OnceLock<Mutex<Option<File>>> = OnceLock::new();

fn enabled() -> bool {
    if DIAGNOSTICS.load(Ordering::Relaxed) == 0 {
        let value = std::env::var("EDEN_DIAGNOSTICS")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes"
                )
            })
            .unwrap_or(false);
        DIAGNOSTICS.store(if value { 2 } else { 1 }, Ordering::Relaxed);
        value
    } else {
        DIAGNOSTICS.load(Ordering::Relaxed) == 2
    }
}

pub struct UiTimingGuard(Option<Instant>);

pub fn begin_ui() -> UiTimingGuard {
    if enabled() {
        UI_CALLS.fetch_add(1, Ordering::Relaxed);
        UiTimingGuard(Some(Instant::now()))
    } else {
        UiTimingGuard(None)
    }
}

impl Drop for UiTimingGuard {
    fn drop(&mut self) {
        if let Some(start) = self.0 {
            UI_NANOS.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
            maybe_report();
        }
    }
}

pub fn record_repaint_request() {
    if enabled() {
        REPAINT_REQUESTS.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_rendered_frame() {
    if enabled() {
        RENDERED_FRAMES.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_viewport_event() {
    if enabled() {
        VIEWPORT_EVENTS.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_region(name: &'static str, elapsed: Duration) {
    if enabled() {
        let timings = REGION_TIMINGS.get_or_init(|| Mutex::new(BTreeMap::new()));
        let mut timings = timings.lock().expect("diagnostic mutex poisoned");
        let entry = timings.entry(name).or_default();
        entry.0 += 1;
        entry.1 += elapsed.as_nanos() as u64;
    }
}

fn maybe_report() {
    let should_report = {
        let now = Instant::now();
        let mut last_report = LAST_REPORT.lock().expect("diagnostic mutex poisoned");
        if last_report.is_none_or(|last| now.duration_since(last) >= Duration::from_secs(5)) {
            *last_report = Some(now);
            true
        } else {
            false
        }
    };
    if should_report {
        let calls = UI_CALLS.load(Ordering::Relaxed);
        let nanos = UI_NANOS.load(Ordering::Relaxed);
        report_line(format!(
            "EdenExplorer diagnostics (totals): ui_calls={calls} ui_ms={:.1} repaint_requests={} rendered_frames={} viewport_events={}",
            nanos as f64 / 1_000_000.0,
            REPAINT_REQUESTS.load(Ordering::Relaxed),
            RENDERED_FRAMES.load(Ordering::Relaxed),
            VIEWPORT_EVENTS.load(Ordering::Relaxed),
        ));
        if let Some(timings) = REGION_TIMINGS.get() {
            let timings = timings.lock().expect("diagnostic mutex poisoned");
            for (name, (region_calls, region_nanos)) in timings.iter() {
                report_line(format!(
                    "EdenExplorer diagnostics region: {name} calls={region_calls} ms={:.1}",
                    *region_nanos as f64 / 1_000_000.0,
                ));
            }
        }
    }
}

fn report_line(line: String) {
    eprintln!("{line}");
    if !diagnostics_requested() {
        return;
    }

    let Some(path) = std::env::var_os("EDEN_DIAGNOSTICS_FILE") else {
        return;
    };
    let output =
        DIAGNOSTIC_OUTPUT.get_or_init(|| Mutex::new(open_diagnostic_file(Path::new(&path))));
    let mut output = output.lock().expect("diagnostic output mutex poisoned");
    if let Some(file) = output.as_mut() {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn open_diagnostic_file(path: &Path) -> Option<File> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "EdenExplorer: failed to create diagnostics directory {:?}: {error}",
            parent
        );
        return None;
    }

    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => Some(file),
        Err(error) => {
            eprintln!(
                "EdenExplorer: failed to open diagnostics file {:?}: {error}",
                path
            );
            None
        }
    }
}
