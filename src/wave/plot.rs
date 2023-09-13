use eframe::egui::{self, Color32, Label, plot::{PlotPoint, BarChart, Bar}, RichText};
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
        Color32::from_rgb(255, 59, 71),   // tomato
        Color32::from_rgb(135, 206, 235), // sky blue
        Color32::from_rgb(0, 255, 127),   // SpringGreen
        Color32::from_rgb(106, 90, 205),  // SlateBlue
    ];
    let mut color_idx = 0;
    let mut idx = 0;
    for (fft_buff, wave_buff) in fft_buffs.iter().zip(wave_buffs.iter()) {
        ui.allocate_ui(space, |ui| {
            ui.columns(2, |columns| {
                // convert numeric data (`f32`) to egui's `Value` struct in order to
                // generate the plots
                let raw_line = Line::new(PlotPoints::from_ys_f32(
                    // reverse buffer
                    wave_buff
                        .iter()
                        .map(|v| *v)
                        .collect::<Vec<f32>>()
                        .as_slice(),
                ))
                .color(colors[color_idx]);

                let num_bins = WAVE_BUFF_LEN as f64 / 2.0;
                let nyquist = SAMPLING_RATE as f64 / 2.0;
                let bin_size = nyquist / num_bins;
                let fft_bars: Vec<Bar> = fft_buff
                    .iter()
                    .enumerate()
                    .map(|(i, v)|{
                        let freq = (i as f64) * bin_size;
                        let mag = *v as f64;
                        Bar::new(freq, mag)
                            .width(0.095)
                            .name(format!("{freq} Hz"))
                         })
                    .take_while(|v| v.argument <= 60.0)
                    .collect();

                // Calculate the frequency bands
                let alpha_mag = frequency_band(&fft_bars, 8.0, 12.0);
                let theta_mag = frequency_band(&fft_bars, 5.0, 7.0);
                let beta_mag = frequency_band(&fft_bars, 13.0, 20.0);

                let fft_barchart = BarChart::new(fft_bars).color(colors[color_idx]);

                if color_idx == colors.len() - 1 {
                    color_idx = 0;
                } else {
                    color_idx += 1;
                }

                columns[0].horizontal_top(|mut ui| {
                    ui.vertical(|ui| {
                        // Channel label
                        let text = RichText::new(format!("CH-{}", idx+1)).strong();
                        ui.label(text);
                        ui.label(format!("alpha: {:.2}%", alpha_mag));
                        ui.label(format!("beta: {:.2}%", beta_mag));
                        ui.label(format!("theta: {:.2}%", theta_mag));
                    });

                    let legend = Legend::default();

                    Plot::new(format!("Raw wave {idx}"))
                        .allow_drag(false)
                        .allow_zoom(false)
                        // .include_y(1)
                        // .center_y_axis(true)
                        .legend(legend)
                        .show_axes([false, false])
                        .show(&mut ui, |plot_ui| plot_ui.line(raw_line));
                });

                Plot::new(format!("FFT {idx}"))
                    .allow_drag(false)
                    .allow_zoom(false)
                    .show_y(false)
                    // .include_y(1024)
                    // .center_y_axis(true)
                    .show(&mut columns[1], |plot_ui| plot_ui.bar_chart(fft_barchart));
            });
        });
        idx += 1;
    }
}

/// Compute the magnitude of a frequency band (bounds included) normalized (percentage) w.r.t the sum of the magnitudes of the FFT.
fn frequency_band(fft: &Vec<Bar>, start: f64, end: f64) -> f64 {
    let fft_mag_sum = fft.iter().map(|v| v.value).sum::<f64>();
    let band_sum =
        fft.iter()
        .filter(|v| v.argument >= start && v.argument <= end)
        .map(|v| v.value)
        .sum::<f64>();
    100.0 * band_sum / fft_mag_sum
}
