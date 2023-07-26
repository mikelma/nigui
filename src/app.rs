#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, epi};

use super::wave;

pub struct MyApp {}

impl Default for MyApp {
    fn default() -> Self {
        Self {}
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();

            egui::widgets::global_dark_light_mode_switch(ui);

            ui.heading("NiGUI: Neural data visualization tool");
            ui.separator();

            wave::plot_waves(ui);
        });

        // resize the native window to be just the size we need it to be
        frame.set_window_size(ctx.used_size());
    }
}
