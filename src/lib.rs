#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;

mod app;
pub mod wave;
pub mod wifi;
pub use app::MyApp;
mod filter_coefs;
