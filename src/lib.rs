#![recursion_limit = "512"]

#[macro_use]
extern crate lazy_static;

mod app;
pub mod wave;
pub mod wifi;
pub use app::MyApp;

use wifi::ERRORS;

pub fn log_err(msg: String) {
    ERRORS.write().unwrap().push(msg);
}
