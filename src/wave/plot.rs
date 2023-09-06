use eframe::egui::{self, Color32, Label, plot::PlotPoint};
use egui::{
    plot::{Legend, Line, Plot, PlotPoints},
    Vec2,
};

use super::*;

/// This function draws the wave and FFT plots. The data used to
/// generate the plots is read from the global variables: `FFT_BUFFS`
/// and `WAVE_BUFFS`.
pub fn plot_waves(ui: &mut egui::Ui) {
    let space = Vec2::from(&[
        ui.available_width(),
        (ui.available_height() / WAVE_BUFFS_NUM as f32) - 5.,
    ]);

    // read the data from the global buffers
    let fft_buffs = FFT_BUFFS.read().unwrap();
    let wave_buffs = WAVE_BUFFS.read().unwrap();

    let colors = vec![
        Color32::RED,
        Color32::GREEN,
        Color32::LIGHT_BLUE,
        Color32::YELLOW,
    ];
    let mut idx = 0;
    for (fft_buff, (_, wave_buff)) in fft_buffs.iter().zip(wave_buffs.iter()) {
        ui.allocate_ui(space, |ui| {
            ui.columns(2, |columns| {
                // convert numeric data (`f32`) to egui's `Value` struct in order to
                // generate the plots
                let raw_line = Line::new(PlotPoints::from_ys_f32(
                    // reverse buffer
                    wave_buff
                        .iter()
                        .rev()
                        .map(|v| *v)
                        .collect::<Vec<f32>>()
                        .as_slice(),
                ))
                .color(colors[idx]);

                let num_bins = WAVE_BUFF_LEN as f64 / 2.0;
                let nyquist = SAMPLING_RATE as f64 / 2.0;
                let bin_size = nyquist / num_bins;
                let fft_values: Vec<[f64; 2]> = fft_buff
                    .iter()
                    .enumerate()
                    .map(|(i, v)| [(i as f64) * bin_size, *v as f64])
                    .take_while(|v| v[0] <= 60.0)
                    .collect();
                let fft_line = Line::new(PlotPoints::new(fft_values));

                if idx == colors.len() - 1 {
                    idx = 0;
                } else {
                    idx += 1;
                }

                columns[0].horizontal_top(|mut ui| {
                    ui.vertical(|ui| {
                        ui.label("Kaixo");
                    });
                    let legend = Legend::default();

                    Plot::new("Raw wave")
                        .allow_drag(false)
                        .allow_zoom(false)
                        // .include_y(1)
                        // .center_y_axis(true)
                        .legend(legend)
                        .show_axes([false, false])
                        .show(&mut ui, |plot_ui| plot_ui.line(raw_line));
                });

                Plot::new("FFT")
                    .allow_drag(false)
                    .allow_zoom(false)
                    .show_y(false)
                    // .include_y(1024)
                    // .center_y_axis(true)
                    .show(&mut columns[1], |plot_ui| plot_ui.line(fft_line));
            });
        });
    }
}
