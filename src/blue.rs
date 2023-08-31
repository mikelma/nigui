use btleplug::api::Characteristic;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use std::error::Error;
use std::fmt;
use std::time::Duration;

use tokio::time;
use uuid::Uuid;

use crate::wave::*;

const BATT_CHAR_UUID: &'static str = "50f9da7d-8dd4-4354-956d-3d1b5d68322e";
const START_STOP_CHAR_UUID: &'static str = "5c804a1f-e0c0-4e30-b068-55a47b8a60c7";
const DATA_CHAR_UUID: &'static str = "9d746b90-552c-4eef-b3f0-506d1b3a48b2";
const NAPSE_DEVICE_NAME: &'static str = "NIT-BL-ECF517A89110";

const MAX_24_BIT: f64 = 0x800000 as f64;

pub struct Napse {
    pub device: Peripheral,
    pub char_batt: Characteristic,
    pub char_start_stop: Characteristic,
    pub char_data: Characteristic,
}

#[derive(Debug)]
pub enum NapseError {
    DeviceNotFound,
    FailedToReadAllChannels,
}

pub async fn create_napse<'a>() -> Result<Napse, Box<dyn Error>> {
    let device = discover_napse().await?;
    let chars = device.characteristics();
    let char_batt = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(BATT_CHAR_UUID).unwrap())
        .unwrap()
        .clone();
    let char_start_stop = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(START_STOP_CHAR_UUID).unwrap())
        .unwrap()
        .clone();
    let char_data = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(DATA_CHAR_UUID).unwrap())
        .unwrap()
        .clone();

    let napse = Napse {
        device,
        // chars,
        char_batt,
        char_start_stop,
        char_data,
    };

    Ok(napse)
}

pub async fn read_to_plots(napse: &Napse) -> Result<(), Box<dyn Error>> {
    // Read the data from Napse (all channels)
    let data_arr = get_data(napse).await?;
    println!("{:?}", data_arr);

    // Write the readed data to the wave buffers
    let mut buffs = WAVE_BUFFS.write()?;
    for (buf_idx, (n, buffer)) in buffs.iter_mut().enumerate() {
        buffer[*n] = data_arr[buf_idx];
        *n = if *n == WAVE_BUFF_LEN - 1 { 0 } else { *n + 1 };
    }
    Ok(())
}

pub async fn get_data<'a>(napse: &'a Napse) -> Result<[f32; 4], Box<dyn Error>> {
    let bytes = napse.device.read(&napse.char_data).await?;

    let data: Vec<i32> = bytes
        .chunks(4)
        .map(|v| i32::from_le_bytes([v[0], v[1], v[2], v[3]]))
        .collect();

    let to_float = |mut v: i32| {
        if v & 0x800000 != 0 {
            v |= !0xffffff;
        }
        (v as f64 / MAX_24_BIT) as f32
    };

    if data.len() != 10 {
        Err(Box::new(NapseError::FailedToReadAllChannels))
    } else {
        Ok([
            to_float(data[2]),
            to_float(data[3]),
            to_float(data[4]),
            to_float(data[5]),
        ])
    }
}

pub async fn send_start_stop(napse: &Napse, start: bool) -> Result<(), Box<dyn Error>> {
    napse
        .device
        .write(
            &napse.char_start_stop,
            &[start.into()],
            WriteType::WithoutResponse,
        )
        .await?;
    Ok(())
}

pub async fn get_battery(napse: &Napse) -> Result<f32, Box<dyn Error>> {
    let res = napse.device.read(&napse.char_batt).await?;
    let res = f32::from_le_bytes([res[0], res[1], res[2], res[3]]);
    Ok(res)
}

pub async fn discover_napse() -> Result<Peripheral, Box<dyn Error>> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = central.peripherals().await?;
    let mut nit = None;
    for p in peripherals {
        if let Some(name) = p.properties().await?.unwrap().local_name {
            if name == NAPSE_DEVICE_NAME {
                nit = Some(p);
            }
            println!("    > {name}");
        }
    }

    match nit {
        Some(napse) => {
            napse.connect().await?;
            // discover services and characteristics
            napse.discover_services().await?;
            Ok(napse)
        }
        None => Err(Box::new(NapseError::DeviceNotFound)),
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
