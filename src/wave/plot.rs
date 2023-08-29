use eframe::egui;
use egui::{
    plot::{Legend, Line, Plot, Values},
    Vec2,
};

use super::*;

/// This function draws the wave and FFT plots. The data used to
/// generate the plots is read from the global variables: `FFT_BUFFS`
/// and `WAVE_BUFFS`.
pub fn plot_waves(ui: &mut egui::Ui) {
    let space = Vec2::from(&[
        ui.available_width(),
        ( ui.available_height() / WAVE_BUFFS_NUM as f32 ) - 5.,
    ]);

    // read the data from the global buffers
    let fft_buffs = FFT_BUFFS.read().unwrap();
    let wave_buffs = WAVE_BUFFS.read().unwrap();

    let legend = Legend::default();

    for (fft_buff, (_, wave_buff)) in fft_buffs.iter().zip(wave_buffs.iter()) {
        ui.allocate_ui(space, |ui| {
            ui.columns(2, |columns| {
                // convert numeric data (`f32`) to egui's `Value` struct in order to
                // generate the plots
                let raw_line = Line::new(Values::from_ys_f32(
                    // reverse buffer
                    wave_buff
                        .iter()
                        .rev()
                        .map(|v| *v)
                        .collect::<Vec<f32>>()
                        .as_slice(),
                ));
                let fft_line = Line::new(Values::from_ys_f32(fft_buff));

                Plot::new("Raw wave")
                    .allow_drag(false)
                    .allow_zoom(false)
                    // .include_y(1)
                    // .center_y_axis(true)
                    .legend(legend)
                    .show(&mut columns[0], |plot_ui| plot_ui.line(raw_line));

                Plot::new("FFT")
                    .allow_drag(false)
                    .allow_zoom(false)
                    // .include_y(1024)
                    // .center_y_axis(true)
                    .show(&mut columns[1], |plot_ui| plot_ui.line(fft_line));

            });
        });
    }
}
