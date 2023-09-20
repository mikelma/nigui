#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::prelude::*;
use eframe::egui;
use eframe::egui::Key;

use super::wave;
use super::wifi::{send_tcp_command, NAPSE_ADDR};
use crate::wave::WAVE_BUFFS_NUM;
use crate::wifi::ERRORS;

pub struct MyApp {
    recording: bool,
    mark_str: String,
    add_str: String,
    test_mode: bool,
    noise_mode: bool,
    impedance_mode: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            recording: false,
            mark_str: String::from("1"),
            add_str: String::from("172.16.30.150"),
            test_mode: false,
            noise_mode: false,
            impedance_mode: false,
        }
    }
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    fn record_button(&mut self, ui: &mut egui::Ui) {
        let text = if self.recording {
            "Stop recording"
        } else {
            "Start recording"
        };

        let mut button = egui::Button::new(text);
        if self.recording {
            button = button.fill(egui::Color32::DARK_RED);
        }

        if ui.add(button).clicked() {
            self.recording = !self.recording;

            {
                let mut flag = crate::wave::RECORDING_FLAG.write().unwrap();
                *flag = self.recording;
            }

            if self.recording {
                let mut buffs = crate::wave::RECORDING_BUFFS.write().unwrap();
                buffs.clear();
                for _ in 0..(WAVE_BUFFS_NUM + 1) {
                    // create an extra buffer for the marks
                    buffs.push(vec![]);
                }
            } else {
                let buffs = crate::wave::RECORDING_BUFFS.read().unwrap();
                write_data_to_file(
                    &format!("recording-{}.csv", Utc::now().to_rfc3339()),
                    buffs.to_vec(),
                );
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();

            let keys = &[Key::Q, Key::W, Key::E, Key::R, Key::T, Key::Y];
            for (i, k) in keys.iter().enumerate() {
                if ctx.input(|i| i.key_pressed(*k)) {
                    println!("Sending mark...🦝 value={}", i + 1);
                    send_tcp_command(0x33, &[(i + 1) as u8]).unwrap();
                }
            }
            ui.heading("NiGUI: Neural data visualization tool");

            ui.horizontal(|ui| {
                ui.label("Napse address: ");
                ui.add(egui::TextEdit::singleline(&mut self.add_str).desired_width(100.0));
                if ui.add(egui::Button::new("Connect ⏩")).clicked() {
                    *NAPSE_ADDR.write().unwrap() = Some(self.add_str.clone());
                }
            });

            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                self.record_button(ui);

                ui.separator();
                ui.label("Mark number: ");
                ui.add(egui::TextEdit::singleline(&mut self.mark_str).desired_width(30.0));

                if ui.add(egui::Button::new("Send mark")).clicked() {
                    println!("Sending mark...🦝 value={}", self.mark_str);
                    let m: u8 = self.mark_str.parse().unwrap(); // TODO: Handle the error better
                    send_tcp_command(0x33, &[m]).unwrap();
                }

                ui.label(" (use QWERTY to send marks 1-6)");

                ui.separator();
                ui.label("Modes: ");
                let mut test_button = egui::Button::new("Test");
                if self.test_mode {
                    test_button = test_button.fill(egui::Color32::DARK_GREEN);
                }
                if ui.add(test_button).clicked() && !self.impedance_mode && !self.noise_mode {
                    self.test_mode = !self.test_mode;
                    if self.test_mode {
                        send_tcp_command(0x77, &[1]).unwrap(); // test ON
                    } else {
                        send_tcp_command(0xaa, &[1]).unwrap(); // test OFF
                    }
                }

                // Noise measurement
                let mut noise_button = egui::Button::new("Noise");
                if self.noise_mode {
                    noise_button = noise_button.fill(egui::Color32::DARK_GREEN);
                }

                if ui.add(noise_button).clicked() && !self.test_mode && !self.impedance_mode {
                    self.noise_mode = !self.noise_mode;
                    if self.noise_mode {
                        send_tcp_command(0x66, &[1]).unwrap();
                    } else {
                        send_tcp_command(0xaa, &[1]).unwrap();
                    }
                }

                // Impedance button
                let mut imp_button = egui::Button::new("Impedance");
                if self.impedance_mode {
                    imp_button = imp_button.fill(egui::Color32::DARK_GREEN);
                }

                if ui.add(imp_button).clicked() && !self.noise_mode && !self.test_mode {
                    self.impedance_mode = !self.impedance_mode;
                    if self.impedance_mode {
                        // TODO
                        // send_tcp_command(0x66, &[1]).unwrap();
                    } else {
                        // TODO
                        // send_tcp_command(0xaa, &[1]).unwrap();
                    }
                }

                /*
                if self.recording && self.timer.elapsed().as_secs() >= 10 {
                    self.timer = Instant::now();
                    send_tcp_command
                }*/
            });
            ui.separator();

            wave::plot_waves(ui);
            let n_errs = ERRORS.read().unwrap().len();
            ui.label("Errors: ");
            if n_errs > 0 {
                {
                    let last = ERRORS.read().unwrap()[0].clone();
                    ui.label(last);
                }
            }
        });

        // resize the native window to be just the size we need it to be
        // frame.set_window_size(ctx.used_size());
    }
}

fn write_data_to_file(fname: &str, bufs: Vec<Vec<f32>>) {
    use std::fs::File;
    use std::io::Write;

    let mut out = File::create(fname).unwrap();
    let num_bufs = bufs.len();
    for i in 0..num_bufs - 1 {
        write!(out, "channel-{},", i).unwrap();
    }
    write!(out, "mark").unwrap();

    write!(out, "\n").unwrap();
    for j in 0..bufs[0].len() {
        // for each data point
        for i in 0..num_bufs - 1 {
            // for each channel
            write!(out, "{},", bufs[i][j]).unwrap();
        }
        write!(out, "{}", bufs[num_bufs - 1][j]).unwrap();
        write!(out, "\n").unwrap();
    }
}
