use rustfft::num_complex::Complex;

use std::thread;
use std::time::Duration;

use super::*;

/// This function fills `WAVE_BUFFS` with artificially generated sine waves.
pub fn sine_wave() {
    let mut f = 5.0;
    {
        let mut buffs = WAVE_BUFFS.write().unwrap();
        for (n, buffer) in buffs.iter_mut() {
            let v = (2.0 * 3.1416 * (1.0 / f) * (*n as f32)).sin();

            buffer[*n] = v;

            *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };

            // for each wave, increment the frequency by 20Hz
            f += 20.0;
        }
    }

    thread::sleep(Duration::from_millis(30));
}

/// Generates the FFTs of the wave buffers.
pub fn fft_gen() {
    let mut fft_buffs = FFT_BUFFS.write().unwrap();
    let wave_buffs = WAVE_BUFFS.read().unwrap();

    for (fft_buff, (_, wave_buff)) in fft_buffs.iter_mut().zip(wave_buffs.iter()) {
        let mut complex: Vec<Complex<f32>> = wave_buff
            .iter()
            .map(|x| Complex { re: *x, im: 0.0f32 })
            .collect();
        FFT.process(&mut complex);

        for (i, d) in complex.iter().skip(1).take(complex.len() / 2).enumerate() {
            fft_buff[i] = d.norm() * *FFT_SCALE;
        }
    }
}
