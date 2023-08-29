use std::net::{UdpSocket, TcpStream, Shutdown};
use std::io::prelude::*;
use std::error::Error;
use std::fmt;
use crate::wave::*;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

const MAX_24_BIT: f64 = 0x800000 as f64;

pub fn read_napse() -> Result<(), Box<dyn Error>> {
    // let mut stream = TcpStream::connect("napse.local:1337")?;
    let mut stream = TcpStream::connect("172.16.30.226:1337")?;
    stream.write(&[0x55])?;
    stream.shutdown(Shutdown::Both).expect("shutdown call failed");

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 40];

    println!("Listening...");

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
        println!("channel data: {:x} {:?}", data[2], channel_data);

        // Write the readed data to the wave buffers
        {
            let mut buffs = WAVE_BUFFS.write()?;
            for (buf_idx, (n, buffer)) in buffs.iter_mut().enumerate() {
                buffer[*n] = channel_data[buf_idx];
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
