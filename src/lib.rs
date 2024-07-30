#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod elabs;

pub use app::TtsApp;
pub use elabs::{Elabs, Voice};
