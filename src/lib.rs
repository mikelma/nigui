#![recursion_limit = "512"]

#[macro_use]
extern crate lazy_static;

mod app;
mod plugins;
pub mod wave;
pub mod wifi;
pub use app::MyApp;

use wifi::{ERRORS, NOTIFICATIONS};

pub fn log_err(msg: String) {
    ERRORS.write().unwrap().push(msg.clone());
    NOTIFICATIONS.write().unwrap().push(msg);
}
