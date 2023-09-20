use gui::{wave, wifi, MyApp};
use std::time::Duration;
// use tokio::runtime::Runtime;

fn main() {
    println!("Starting UI... 🧠🦝🐙🐰");

    std::thread::spawn(|| {
        wifi::read_napse().unwrap(); // read data in a loop
    });

    std::thread::spawn(|| {
        loop {
            wave::read::fft_gen(); // generate the FFTs of the waves that we have just read
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    // execute GUI
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NiGUI",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    ).expect("Failed to initialize egui GUI");
}
