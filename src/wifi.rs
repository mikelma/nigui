use crate::{wave::*, log_err};
use biquad::*;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream, UdpSocket};
use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant};

lazy_static! {
    pub static ref NAPSE_ADDR: RwLock<Option<String>> = RwLock::new(None);

    pub static ref PRE_BUFFS: RwLock<Vec<Vec<f32>>> = {
        let buf = vec![vec![]; WAVE_BUFFS_NUM];
        RwLock::new(buf)
    };

    pub static ref CH_STATUS: RwLock<[bool; WAVE_BUFFS_NUM]> = RwLock::new([true; WAVE_BUFFS_NUM]);

    pub static ref ERRORS: RwLock<Vec<String>> = RwLock::new(vec![]);
}

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

pub fn send_tcp_command(cmd: u8, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let pld = payload.to_vec();
    let addr = NAPSE_ADDR.read().unwrap().clone().unwrap();
    let x = std::thread::spawn(move || -> Result<(), std::io::Error> {
        let mut stream = TcpStream::connect(&format!("{}:1337", addr))?;
        stream.write(&[&[cmd], pld.as_slice()].concat())?;
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    });

    Ok(x.join().unwrap()?)
}

fn buffer_sync_loop() {
    let wait_time = 1.0 / SAMPLING_RATE as f64;
    let wait = Duration::from_secs_f64(wait_time);
    let mut lasts = vec![0.0; WAVE_BUFFS_NUM];
    let mut val;

    // Cutoff and sampling frequencies
    let f0 = 40.hz();
    let fs = SAMPLING_RATE.hz();

    // Create coefficients for the biquads
    let coeffs =
        Coefficients::<f32>::from_params(Type::LowPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();

    // Initialize one filter for each channel
    let mut filters = vec![];
    for _ in 0..WAVE_BUFFS_NUM {
        filters.push(DirectForm1::<f32>::new(coeffs));
    }

    loop {
        let now = Instant::now();
        {
            let mut pre_buf = PRE_BUFFS.write().unwrap();
            let mut wave_buf = WAVE_BUFFS.write().unwrap();
            for (buf_idx, buff) in wave_buf.iter_mut().enumerate() {
                val = match pre_buf[buf_idx].pop() {
                    Some(v) => v,
                    None => lasts[buf_idx],
                };

                val = filters[buf_idx].run(val);

                // Update the buffer
                for i in 0..(WAVE_BUFF_LEN - 1) {
                    buff[i] = buff[i + 1];
                }
                buff[WAVE_BUFF_LEN - 1] = val;

                // store the current value as the last value
                lasts[buf_idx] = val;

                // Update the pointer
                // *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };
            }
        }
        let elapsed_time = now.elapsed();
        if elapsed_time < wait {
            thread::sleep(wait - elapsed_time);
        } else {
            log_err("Buffer out of sync".into());
        }
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

    send_tcp_command(0xdd, &[])?; // send impedance ON command

    // start buffer synchronization
    thread::spawn(|| {
        buffer_sync_loop();
    });

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 44];

    println!("Listening...");

    let mut time_start = Instant::now();
    let mut _n_pkgs = 0;
    let mut ch_status = vec![false; WAVE_BUFFS_NUM];
    loop {
        let (_amt, _src) = socket.recv_from(&mut buf)?;

        let data: Vec<i32> = buf
            .chunks(4)
            .skip(2)
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

        if data.len() != 9 {
            return Err(Box::new(NapseError::FailedToReadAllChannels));
        }

        // get channel status
        let status_data = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        for i in 0..WAVE_BUFFS_NUM {
            let not_stat = (status_data & (1 << 12 + i)) >> (12 + i);
            ch_status[i] = if not_stat == 1 { false } else { true} ;
        }
        {
            let mut status_array = CH_STATUS.write().unwrap();
            for i in 0..WAVE_BUFFS_NUM {
                status_array[i] = ch_status[i];
            }
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
                        rec_buf[WAVE_BUFFS_NUM + buf_idx].push(if ch_status[buf_idx] { 1.0 } else { 0.0 });

                        if buf_idx == WAVE_BUFFS_NUM - 1 {
                            let mark = buf[40];
                            let n = rec_buf.len()-1;
                            rec_buf[n].push(mark as f32);
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
        _n_pkgs += 1;
        if time_start.elapsed().as_millis() >= 1000 {
            // println!("*** num packages: {}", n_pkgs);
            time_start = Instant::now();
            _n_pkgs = 0;
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
