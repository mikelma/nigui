use std::net::{UdpSocket, TcpStream, Shutdown};
use std::io::prelude::*;
use std::error::Error;
use std::fmt;
use crate::wave::*;
use std::time::{Duration, Instant};
// use synthrs::filter::{convolve, cutoff_from_frequency, lowpass_filter};
// use biquad::*;

use digital_filter::DigitalFilter;
use generic_array::*;

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

const MAX_24_BIT: f64 = 0x800000 as f64;

pub fn read_napse() -> Result<(), Box<dyn Error>> {
    // let mut stream = TcpStream::connect("napse.local:1337")?;
    let mut stream = TcpStream::connect("172.16.30.150:1337")?;
    stream.write(&[0x55])?;
    stream.shutdown(Shutdown::Both).expect("shutdown call failed");

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 40];

    println!("Listening...");

    // create the low pass filter for 50Hz
    // let lowpass = lowpass_filter(cutoff_from_frequency(50.0, 250), 0.01);
    // let lowpass = lowpass.iter().map(|x| *x as f32).collect::<Vec<f32>>();
    // let filter_size = lowpass.len();

    // Cutoff and sampling frequencies
    // let f0 = 10.hz();
    // let fs = 500.hz();
    // Create coefficients for the biquads
    // let coeffs = Coefficients::<f32>::from_params(Type::LowPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
    // Create two different biquads
    // let mut biquad = DirectForm1::<f32>::new(coeffs);

    // coefficients generated with: http://t-filter.engineerjs.com/
    let coefs = arr![f32;
-0.0007523975118029623,
  -0.01094960581783574,
  -0.013842869818593638,
  -0.019926898866304207,
  -0.024575048312386424,
  -0.026607040840587196,
  -0.024906724051053646,
  -0.019070608432511366,
  -0.009692873094479401,
  0.00163083161860868,
  0.01248463805323288,
  0.020194394551224685,
  0.022466048704565873,
  0.01807294737447773,
  0.007400878265218106,
  -0.0074172324398747635,
  -0.022726929577830672,
  -0.03402502688062672,
  -0.036950563532114435,
  -0.0283416109266146,
  -0.007112910502064243,
  0.02525613213838398,
  0.064770881822736,
  0.1055840808172426,
  0.14108743018349548,
  0.16522897170246068,
  0.1737814385064219,
  0.16522897170246068,
  0.14108743018349548,
  0.1055840808172426,
  0.064770881822736,
  0.02525613213838398,
  -0.007112910502064243,
  -0.0283416109266146,
  -0.036950563532114435,
  -0.03402502688062672,
  -0.022726929577830672,
  -0.0074172324398747635,
  0.007400878265218106,
  0.01807294737447773,
  0.022466048704565873,
  0.020194394551224685,
  0.01248463805323288,
  0.00163083161860868,
  -0.009692873094479401,
  -0.019070608432511366,
  -0.024906724051053646,
  -0.026607040840587196,
  -0.024575048312386424,
  -0.019926898866304207,
  -0.013842869818593638,
  -0.01094960581783574,
  -0.0007523975118029623
];


    let mut filters = vec![];
    for _ in 0..WAVE_BUFFS_NUM {
        filters.push(DigitalFilter::new(coefs));
    }

    let mut time_start = Instant::now();
    let mut n_pkgs = 0;
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        // println!("> Someone @ {src} send {amt} bytes like: {:?}", buf);

        let data: Vec<i32> = buf
            .chunks(4)
            .map(|v| {
                i32::from_le_bytes([v[0], v[1], v[2], v[3]])
            })
            .collect();

        let to_float = |mut v: i32| {
            if v & 0x800000 != 0 {
                // v |= (!0xff000000u32) as i32;
                v |= -16777216i32;
            }

            // (v as f64 / MAX_24_BIT) as f32 - 1.

            let min = -8388608.0_f64;
            let max = 8388607.0_f64;

            // println!("min: {}, max: {}", min, max);
            // panic!();

            let val = v as f64;
            let v = ((val - min) / (max - min)) as f32;
            2.0*v - 1.0

            // v as f32
        };

        if data.len() != 10 {
            return Err(Box::new(NapseError::FailedToReadAllChannels));
        }

        let channel_data = [to_float(data[2]), to_float(data[3]), to_float(data[4]), to_float(data[5])];
        // println!("channel data: {:x} {:?}", data[2], channel_data);

        // Write the readed data to the wave buffers
        {
            let mut buffs = WAVE_BUFFS.write()?;
            for (buf_idx, (n, buffer)) in buffs.iter_mut().enumerate() {
                // buffer[*n] = channel_data[buf_idx];  // ===> ORIGINAL

                let val = channel_data[buf_idx];
                // buffer[*n] = biquad.run(val);
                buffer[*n] = filters[buf_idx].filter(val);
                // buffer[*n] = val;

                {
                    let record_flag = RECORDING_FLAG.read().unwrap();
                    if *record_flag {
                        let mut rec_buf = RECORDING_BUFFS.write().unwrap();
                        rec_buf[buf_idx].push(buffer[*n]);
                    }
                }

                /* ======================================
                // Apply the filter per channel
                let delay_buf = if *n >= filter_size {
                    //println!("Case 1) n={n}");
                    buffer[(*n-filter_size)..*n].to_vec()
                } else {
                    let m = buffer.len();
                    let b = &buffer[0..=*n];
                    let a = &buffer[(m - (filter_size - b.len()))..];
                    // println!("Case 2) n={n}, a={}, b={}", a.len(), b.len());
                    let ab = [a, b].concat();
                    ab
                };

                let new_val: f32 = std::iter::zip(&delay_buf, &lowpass).map(|(x, y)| x*y).sum();
                buffer[*n] = new_val;
                =========================================== */

                // Update the pointer
                *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };
            }
        }

        // Clear buffer
        for elem in buf.iter_mut() { *elem = 0; }

        n_pkgs += 1;

        if time_start.elapsed().as_millis() >= 1000 {
            println!("*** num packages: {}", n_pkgs);
            time_start = Instant::now();
            n_pkgs = 0;
        }
    }
}

/*
fn filter_signal(data: &mut [f32], fs: usize, cutof_freq: f32) {
    for i in 0..data.len() {
        data[i] =
    }
}

fn filter_alpha(cutof_freq: f32) -> f32 {
    let rc = 1.0 / (cutof_freq * 2 * 3.1416);
    let dt = 1.0 / 250; // HACK: 250 is the sample rate
    let alpha = dt / (rc + dt);
    return alpha;
}*/

impl fmt::Display for NapseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NapseError::DeviceNotFound => write!(f, "Napse device not found"),
            NapseError::FailedToReadAllChannels => write!(f, "Failed to read all the channels of the Napse device"),
        }
    }
}

impl Error for NapseError {}
