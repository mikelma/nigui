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

/*
pub fn serial_read<T: SerialPort>(port: &mut T) {
    let mut bytes = vec![0u8; 255];

    if let Err(err) = port.read(&mut bytes) {
        eprintln!("[WARNING] Cannot read from serial port: {err}");
        return;
    }
    /*
    match port.read(&mut bytes) {
        Ok(n) => {
            println!("read {} bytes from serial", n);
        },
        Err(err) => {
            eprintln!("[WARNING] Cannot read from serial port: {err}");
            return;
        }
    }
    */

    let string_data = match String::from_utf8(bytes) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("[WARNING] Cannot decode serial data as UTF-8: {err}");
            return;
        }
    };

    let parsed = string_data
        .split_terminator("\n")
        .filter_map(|v| v.trim().parse::<f32>().ok())
        .collect::<Vec<f32>>();
    {
        let mut buffs = WAVE_BUFFS.write().unwrap();
        let (n, buffer) = &mut buffs[0]; // get the first buffer

        for data in parsed {
            buffer[*n] = data;
            // update circular buffer's index
            *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };
        }

        // ======= filter ======= //
        /*
        let chunk_size = 2;
        let scale = 1.0/chunk_size as f32;
        for i in chunk_size-1..WAVE_BUFF_LEN {
            buffer[i] = buffer[i-(chunk_size-1)..i+1].iter().sum::<f32>() * scale;
        }
        */
        let mut i = 0;
        for _ in 0..WAVE_BUFF_LEN / 2 {
            if (buffer[i] - buffer[i + 1]).abs() > 300.0 {
                buffer[i] = buffer[i + 1];
            }
            i += 2;
        }
        // ====================== //
    }
}
*/

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

        for (i, d) in complex.iter().enumerate() {
            fft_buff[i] = d.norm()**FFT_SCALE;
        }
    }
}
