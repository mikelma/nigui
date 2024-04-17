#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::prelude::*;
use eframe::egui::{self, RichText};
use eframe::egui::Key;
use egui::ColorImage;
use rfd::FileDialog;

use super::wave;
use super::wifi::{send_tcp_command, NAPSE_ADDR};
use crate::wave::WAVE_BUFFS_NUM;
use crate::wifi::{ERRORS, MARKER_ADDR};

pub struct MyApp {
    recording: bool,
    mark_str: String,
    add_str: String,
    test_mode: bool,
    noise_mode: bool,
    impedance_mode: bool,
    marker_addr: String,
    marker_connected: bool,

    logo_tex: Option<egui::TextureHandle>,
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
            logo_tex: None,
            marker_addr: String::from("127.0.0.1:20001"),
            marker_connected: false,
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

        let is_connected = NAPSE_ADDR.read().unwrap().is_some();

        if ui.add(button).clicked() {
            if !self.recording && is_connected {
                self.recording = true;
            } else if self.recording {
                self.recording = false;
            }

            {
                let mut flag = crate::wave::RECORDING_FLAG.write().unwrap();
                *flag = self.recording;
            }

            if self.recording {
                let mut buffs = crate::wave::RECORDING_BUFFS.write().unwrap();
                buffs.clear();
                // push a vec for each column in the CSV
                for _ in 0..(WAVE_BUFFS_NUM*2 + 1) {
                    buffs.push(vec![]);
                }
            } else {
                let buffs = crate::wave::RECORDING_BUFFS.read().unwrap();
                let default = format!("recording-{}.csv", Utc::now().to_rfc3339());
                let file = FileDialog::new()
                    .set_file_name(&default)
                    .save_file();

                if let Some(path) = file {
                    write_data_to_file(
                        &path.to_string_lossy().to_string(),
                        buffs.to_vec(),
                    );
                }
            }
        }
    }
}


fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let connected = NAPSE_ADDR.read().unwrap().is_some();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();

            let keys = &[Key::Q, Key::W, Key::E, Key::R, Key::T, Key::Y];
            for (i, k) in keys.iter().enumerate() {
                if ctx.input(|i| i.key_pressed(*k)) && connected {
                    println!("Sending mark...🦝 value={}", i + 1);
                    send_tcp_command(0x33, &[(i + 1) as u8]).unwrap();
                }
            }

            let texture: &egui::TextureHandle = self.logo_tex.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx().load_texture(
                    "logo-img",
                    load_image_from_memory(include_bytes!("../logo.png")).unwrap(),
                    Default::default()
                )
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("NiGUI").size(20.0).strong());
                    ui.label(RichText::new("Neural data visualization tool").size(15.0));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.image(texture);
                });
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Napse address: ");
                ui.add(egui::TextEdit::singleline(&mut self.add_str).desired_width(100.0));
                if ui.add(egui::Button::new("Connect ⏩")).clicked() {
                    *NAPSE_ADDR.write().unwrap() = Some(self.add_str.clone());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Marker addr: ");
                let text_edit = egui::TextEdit::singleline(&mut self.marker_addr)
                    .desired_width(100.0)
                    .hint_text("addr:port");
                ui.add(text_edit);

                let text = if !self.marker_connected { "Enable" } else { "Disable" };

                if ui.add(egui::Button::new(text).selected(self.marker_connected)).clicked() {
                    if !self.marker_connected {
                        *MARKER_ADDR.write().unwrap() = Some(self.marker_addr.clone());
                    } else {
                        *MARKER_ADDR.write().unwrap() = None;
                    }
                    self.marker_connected = !self.marker_connected;
                }
            });

            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                self.record_button(ui);

                ui.separator();
                ui.label("Mark number: ");
                ui.add(egui::TextEdit::singleline(&mut self.mark_str).desired_width(30.0));

                if ui.add(egui::Button::new("Send mark")).clicked() && connected {
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
                if ui.add(test_button).clicked() && !self.impedance_mode && !self.noise_mode && connected {
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

                if ui.add(noise_button).clicked() && !self.test_mode && !self.impedance_mode && connected {
                    self.noise_mode = !self.noise_mode;
                    if self.noise_mode {
                        send_tcp_command(0x66, &[1]).unwrap();
                    } else {
                        send_tcp_command(0xaa, &[1]).unwrap();
                    }
                }

                /*
                // Impedance button
                let mut imp_button = egui::Button::new("Impedance");
                if self.impedance_mode {
                    imp_button = imp_button.fill(egui::Color32::DARK_GREEN);
                }

                if ui.add(imp_button).clicked() && !self.noise_mode && !self.test_mode && connected {
                    self.impedance_mode = !self.impedance_mode;
                    if self.impedance_mode {
                        // TODO
                        // send_tcp_command(0x66, &[1]).unwrap();
                    } else {
                        // TODO
                        // send_tcp_command(0xaa, &[1]).unwrap();
                    }
                }*/

                /*
                if self.recording && self.timer.elapsed().as_secs() >= 10 {
                    self.timer = Instant::now();
                    send_tcp_command
                }*/
            });
            ui.separator();

            wave::plot_waves(ui);

            ui.separator();
            ui.horizontal(|ui| {
                let n_errs = ERRORS.read().unwrap().len();

                if n_errs > 0 {
                    ui.label(egui::RichText::new(format!("Messages ({}):", n_errs)).strong());
                    {
                        let last = ERRORS.read().unwrap()[0].clone();
                        ui.label(last);
                    }
                } else {
                    ui.label("Messages: ");
                }
            });
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
    for i in 0..WAVE_BUFFS_NUM {
        write!(out, "channel-{},", i).unwrap();
    }

    for i in 0..WAVE_BUFFS_NUM {
        write!(out, "status ch-{},", i).unwrap();
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
