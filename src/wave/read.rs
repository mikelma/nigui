use super::*;
use rustfft::num_complex::Complex;

/// Generates the FFTs of the wave buffers.
pub fn fft_gen() {
    let mut fft_buffs = FFT_BUFFS.write().unwrap();
    let wave_buffs = WAVE_BUFFS.read().unwrap();

    for (fft_buff, wave_buff) in fft_buffs.iter_mut().zip(wave_buffs.iter()) {
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
