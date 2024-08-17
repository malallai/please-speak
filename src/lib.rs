#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod elabs;
mod errors;
mod device;

pub use app::TtsApp;
pub use elabs::{Elabs, Voice};
pub use errors::ErrorManager;
