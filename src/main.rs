use gui::{wave, wifi, MyApp};
use std::time::Duration;
// use tokio::runtime::Runtime;


fn main() {
    // ===================== Wifi ===================== //
    std::thread::spawn(|| {
        wifi::read_napse().unwrap(); // read data in a loop
    });

    std::thread::spawn(|| {
        loop {
            wave::read::fft_gen(); // generate the FFTs of the waves that we have just read
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    /*
    // ===================== Bluetooth ===================== //
    println!("[INFO] Trying to connect to Napse 🧿...");
    let napse = blue::create_napse().await.unwrap();

    println!("[INFO] Connected to Napse! 🧠⚡");

    let batt = blue::get_battery(&napse).await.unwrap();
    println!("[INFO] NAPSE Battery 🔋 {batt}V");

    // Notify Napse to start sending the data
    blue::send_start_stop(&napse, true).await.unwrap();
    // let data = blue::get_data(&napse).await.unwrap();
    // blue::send_start_stop(&napse, false).await.unwrap();

    tokio::spawn(async move {
        // Call the functions to read the waves and to generate
        // the FFT's of the waves in an infinite loop.
        loop {
            let res = blue::read_to_plots(&napse).await;
            if let Err(e) = res {
                eprintln!("⚠️ Failed to read naural data: {e}");
            }
            // wave::read::serial_read(&mut port);
            // wave::read::sine_wave(); // this function artificially generates sine waves (only for test purposes)
            wave::read::fft_gen(); // generate the FFTs of the waves that we have just read
        }
    });
    */

    /*
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async move {
        loop {
        let res = blue::read_to_plots(&napse).await;
        if let Err(e) = res {
            eprintln!("⚠️ Failed to read naural data: {e}");
        }
        }
    });*/

    // loop {} // NOTE: Just for debugging

    // execute GUI
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::default()), options);
}

/*
async fn test_bluetooth() -> Result<(), Box<dyn std::error::Error>> {
    use btleplug::api::{
        bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
    };
    use btleplug::platform::{Adapter, Manager, Peripheral};
    use std::time::Duration;
    use tokio::time;

    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    let peripherals = central.peripherals().await.unwrap();
    let mut nit = None;
    println!("[INFO] Finding devices...");
    for p in peripherals {
        if let Some(name) = p.properties().await.unwrap().unwrap().local_name {
            if name == "NIT-BL-ECF517A89110" {
                nit = Some(p);
            }
            println!("    > {name}");
        }
    }

    let nit = match nit {
        Some(p) => p,
        None => panic!("No NIT device found! =("),
    };

    nit.connect().await?;

    println!("[INFO] Connected to Napse! 🧠⚡");

    // discover services and characteristics
    nit.discover_services().await?;

    // find the characteristic we want
    let chars = nit.characteristics();

    // read NAPSE's battery
    let char_batt = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(BATT_CHAR_UUID).unwrap())
        .unwrap();
    let res = nit.read(&char_batt).await?;
    let res = f32::from_le_bytes([res[0], res[1], res[2], res[3]]);
    println!("[INFO] NAPSE Battery 🔋 {res}V");

    let char_start_stop = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(START_STOP_CHAR_UUID).unwrap())
        .unwrap();
    let char_data = chars
        .iter()
        .find(|c| c.uuid == Uuid::parse_str(DATA_CHAR_UUID).unwrap())
        .unwrap();

    nit.write(&char_start_stop, &[0x1], WriteType::WithoutResponse)
        .await?;

    let res = nit.read(&char_data).await?;
    dbg!(res);

    time::sleep(Duration::from_secs(1)).await;
    nit.write(&char_start_stop, &[0x0], WriteType::WithoutResponse)
        .await?;

    Ok(())
}
*/
