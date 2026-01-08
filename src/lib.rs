pub mod config;
pub mod error;

pub use error::{RcleanerError, Result};

pub mod backup;
pub mod cleaner;
pub mod models;
pub mod safety;
pub mod system;
pub mod tui;
pub mod utils;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "rCleaner";
