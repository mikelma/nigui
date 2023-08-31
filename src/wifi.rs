use crate::wave::*;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream, UdpSocket};
use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant};

use digital_filter::DigitalFilter;

lazy_static! {
    pub static ref NAPSE_ADDR: RwLock<Option<String>> = { RwLock::new(None) };
}

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

pub fn send_tcp_command(cmd: u8, payload: &str) -> Result<(), Box<dyn Error>> {
    // let mut stream = TcpStream::connect("napse.local:1337")?;
    let pld = payload.to_string();
    let addr = NAPSE_ADDR.read().unwrap().clone().unwrap();
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(&format!("{}:1337", addr)).unwrap();
        let str_bytes = pld.as_bytes();
        stream.write(&[&[cmd], str_bytes].concat()).unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    });

    Ok(())
}

pub fn read_napse() -> Result<(), Box<dyn Error>> {
    println!("Waiting to press play...");
    loop {
        {
            let addr = NAPSE_ADDR.read().unwrap();
            if addr.is_some() {
                break;
            }
        }

        thread::sleep(Duration::from_millis(500));
    }

    send_tcp_command(0x55, "")?; // send start command

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 40];

    println!("Listening...");

    // create the low pass filter for 50Hz
    let mut filters = vec![];
    for _ in 0..WAVE_BUFFS_NUM {
        filters.push(DigitalFilter::new(crate::filter_coefs::coefs()));
    }

    let mut time_start = Instant::now();
    let mut n_pkgs = 0;
    loop {
        let (_amt, _src) = socket.recv_from(&mut buf)?;
        // println!("> Someone @ {src} send {amt} bytes like: {:?}", buf);

        let data: Vec<i32> = buf
            .chunks(4)
            .map(|v| i32::from_le_bytes([v[0], v[1], v[2], v[3]]))
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
            2.0 * v - 1.0

            // v as f32
        };

        if data.len() != 10 {
            return Err(Box::new(NapseError::FailedToReadAllChannels));
        }

        let channel_data = [
            to_float(data[2]),
            to_float(data[3]),
            to_float(data[4]),
            to_float(data[5]),
        ];
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
        for elem in buf.iter_mut() {
            *elem = 0;
        }

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
            NapseError::FailedToReadAllChannels => {
                write!(f, "Failed to read all the channels of the Napse device")
            }
        }
    }
}

impl Error for NapseError {}
