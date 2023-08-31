#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, epi};
use chrono::prelude::*;

use crate::wave::WAVE_BUFFS_NUM;
use super::wave;

pub struct MyApp {
    recording: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self { recording: false }
    }
}

impl MyApp {
    fn record_button(&mut self, ui: &mut egui::Ui) {
        let text = if self.recording {
            "Stop recording"
        } else {
            "Start recording"
        };

        if ui.add(egui::Button::new(text)).clicked() {
            self.recording = !self.recording;

            {
                let mut flag = crate::wave::RECORDING_FLAG.write().unwrap();
                *flag = self.recording;
            }

            if self.recording {
                let mut buffs = crate::wave::RECORDING_BUFFS.write().unwrap();
                buffs.clear();
                for _ in 0..WAVE_BUFFS_NUM {
                    buffs.push(vec![]);
                }
            } else {
                let buffs = crate::wave::RECORDING_BUFFS.read().unwrap();
                write_data_to_file(&format!("recording-{}.csv", Utc::now().to_rfc3339()), buffs.to_vec());
            }
        }
    }
}

fn write_data_to_file(fname: &str, bufs: Vec<Vec<f32>>) {
    use std::fs::File;
    use std::io::Write;

    let mut out = File::create(fname).unwrap();
    let num_channels = bufs.len();
    for i in 0..num_channels - 1 {
        write!(out, "channel-{},", i).unwrap();
    }
    write!(out, "channel-{}", num_channels - 1).unwrap();

    write!(out, "\n").unwrap();
    for j in 0..bufs[0].len() {
        // for each data point
        for i in 0..num_channels - 1 {
            // for each channel
            write!(out, "{},", bufs[i][j]).unwrap();
        }
        write!(out, "{}", bufs[num_channels - 1][j]).unwrap();
        write!(out, "\n").unwrap();
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "NIGUI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();

            egui::widgets::global_dark_light_mode_switch(ui);
            ui.heading("NiGUI: Neural data visualization tool");

            self.record_button(ui);

            ui.separator();

            wave::plot_waves(ui);
        });

        // resize the native window to be just the size we need it to be
        frame.set_window_size(ctx.used_size());
    }
}
