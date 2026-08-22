#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{NativeOptions, Renderer, egui};

struct RendererBench;

impl eframe::App for RendererBench {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("EdenExplorer renderer test");
    }
}

fn renderer_from_environment() -> Renderer {
    match std::env::var("EDEN_RENDERER")
        .unwrap_or_else(|_| "auto".to_owned())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "glow" => Renderer::Glow,
        "wgpu" | "auto" | "" => Renderer::Wgpu,
        invalid => {
            eprintln!("renderer_bench: invalid EDEN_RENDERER={invalid:?}; using wgpu");
            Renderer::Wgpu
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        renderer: renderer_from_environment(),
        viewport: egui::ViewportBuilder::default()
            .with_title("EdenExplorer renderer test")
            .with_inner_size([800.0, 600.0])
            .with_decorations(true)
            .with_title_shown(true),
        ..Default::default()
    };

    eframe::run_native(
        "EdenExplorer renderer test",
        options,
        Box::new(|_cc| Ok(Box::new(RendererBench))),
    )
}
