use std::net::UdpSocket;
use std::error::Error;
use std::fmt;
use crate::wave::*;

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

const MAX_24_BIT: f64 = 0x800000 as f64;

pub fn read_napse() -> Result<(), Box<dyn Error>> {
    // let mut stream = TcpStream::connect("192.168.1.:34254")?;

    let socket = UdpSocket::bind("0.0.0.0:31337")?;

    let mut buf = [0; 40];

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
                v |= !0xffffff;
            }
            (v as f64 / MAX_24_BIT) as f32
        };

        if data.len() != 10 {
            return Err(Box::new(NapseError::FailedToReadAllChannels));
        }

        let channel_data = [to_float(data[2]), to_float(data[3]), to_float(data[4]), to_float(data[5])];
        // println!("channel data: {:?}", channel_data);

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
    }
    Ok(())
}

impl fmt::Display for NapseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NapseError::DeviceNotFound => write!(f, "Napse device not found"),
            NapseError::FailedToReadAllChannels => write!(f, "Failed to read all the channels of the Napse device"),
        }
    }
}

impl Error for NapseError {}
