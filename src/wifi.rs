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
    pub static ref NAPSE_ADDR: RwLock<Option<String>> = RwLock::new(None);

    pub static ref PRE_BUFFS: RwLock<Vec<Vec<f32>>> = {
        let buf = vec![vec![]; WAVE_BUFFS_NUM];
        RwLock::new(buf)
    };
}

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

pub fn send_tcp_command(cmd: u8, payload: &[u8]) -> Result<(), Box<dyn Error>> {
    let pld = payload.to_vec();
    let addr = NAPSE_ADDR.read().unwrap().clone().unwrap();
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(&format!("{}:1337", addr)).unwrap();
        stream.write(&[&[cmd], pld.as_slice()].concat()).unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    });

    Ok(())
}

fn buffer_sync_loop() {
    let wait_time = 1.0 / SAMPLING_RATE as f64;
    let wait = Duration::from_secs_f64(wait_time);
    let mut lasts = vec![0.0; WAVE_BUFFS_NUM];
    let mut val = 0.0;
    loop {
        {
            let mut pre_buf = PRE_BUFFS.write().unwrap();
            let mut wave_buf = WAVE_BUFFS.write().unwrap();
            for (buf_idx, (n, buff)) in wave_buf.iter_mut().enumerate() {
                val = match pre_buf[buf_idx].pop() {
                    Some(v) => v,
                    None => lasts[buf_idx],
                };

                buff[*n] = val;
                lasts[buf_idx] = val;

                // Update the pointer
                *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };
            }
        }
        thread::sleep(wait);
    }
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

    send_tcp_command(0x55, &[])?; // send start command

    // start buffer synchronization
    thread::spawn(|| {
        buffer_sync_loop();
    });

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 44];

    println!("Listening...");

    let mut time_start = Instant::now();
    let mut n_pkgs = 0;
    loop {
        let (_amt, _src) = socket.recv_from(&mut buf)?;

        let data: Vec<i32> = buf
            .chunks(4)
            .map(|v| i32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            .collect();

        let to_float = |mut v: i32| {
            if v & 0x800000 != 0 {
                v |= -16777216i32;
            }

            let min = -8388608.0_f64;
            let max = 8388607.0_f64;

            let val = v as f64;
            let v = ((val - min) / (max - min)) as f32;
            2.0 * v - 1.0
        };

        if data.len() != 11 {
            return Err(Box::new(NapseError::FailedToReadAllChannels));
        }

        let channel_data = [
            to_float(data[2]),
            to_float(data[3]),
            to_float(data[4]),
            to_float(data[5]),
        ];

        // Write the readed data to the wave buffers
        {
            let mut buffs = PRE_BUFFS.write()?;
            for (buf_idx, buffer) in buffs.iter_mut().enumerate() {
                let val = channel_data[buf_idx];
                buffer.push(val);

                // Wave recording
                {
                    let record_flag = RECORDING_FLAG.read().unwrap();
                    if *record_flag {
                        let mut rec_buf = RECORDING_BUFFS.write().unwrap();
                        rec_buf[buf_idx].push(val);

                        if buf_idx == WAVE_BUFFS_NUM-1 {
                            let mark = buf[40];
                            rec_buf[WAVE_BUFFS_NUM].push(mark as f32);
                        }
                    }
                }
            }
        }

        // Clear buffer
        for elem in buf.iter_mut() {
            *elem = 0;
        }

        // Package counting
        n_pkgs += 1;
        if time_start.elapsed().as_millis() >= 1000 {
            // println!("*** num packages: {}", n_pkgs);
            time_start = Instant::now();
            n_pkgs = 0;
        }
    }
}

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
