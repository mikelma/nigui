#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::prelude::*;
use eframe::{egui, epi};

use super::wave;
use super::wifi::{send_tcp_command, NAPSE_ADDR};
use crate::wave::WAVE_BUFFS_NUM;

pub struct MyApp {
    recording: bool,
    mark_str: String,
    add_str: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            recording: false,
            mark_str: String::from("1"),
            add_str: String::from("172.16.30.150"),
        }
    }
}

impl MyApp {
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
                for _ in 0..(WAVE_BUFFS_NUM+1) {  // create an extra buffer for the marks
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

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "NIGUI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();


            /*
            let shorcut = egui::KeyboardShorcut::new(egui::Modifiers::CTRL, Key::);
            if ctx.input(|i| i.consume_shorcut()) {
                self.text.push_str("\nPressed");
            }
             */

            /*
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Num2) {
                }
            });*/


            ui.heading("NiGUI: Neural data visualization tool");

            ui.horizontal(|ui| {
                ui.label("Napse address: ");
                ui.add(egui::TextEdit::singleline(&mut self.add_str).desired_width(100.0));
                if ui.add(egui::Button::new("Play")).clicked() {
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
            });

            ui.separator();

            wave::plot_waves(ui);
        });

        // resize the native window to be just the size we need it to be
        frame.set_window_size(ctx.used_size());
    }
}

fn write_data_to_file(fname: &str, bufs: Vec<Vec<f32>>) {
    use std::fs::File;
    use std::io::Write;

    let mut out = File::create(fname).unwrap();
    let num_bufs = bufs.len();
    for i in 0..num_bufs-1 {
        write!(out, "channel-{},", i).unwrap();
    }
    write!(out, "mark").unwrap();

    write!(out, "\n").unwrap();
    for j in 0..bufs[0].len() {
        // for each data point
        for i in 0..num_bufs-1 {
            // for each channel
            write!(out, "{},", bufs[i][j]).unwrap();
        }
        write!(out, "{}", bufs[num_bufs - 1][j]).unwrap();
        write!(out, "\n").unwrap();
    }
}
