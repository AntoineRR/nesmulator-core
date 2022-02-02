//! This crate provides an API to run a NES emulator.
//! See more about the project on [Github](https://github.com/AntoineRR/nesmulator).

/// Contain the NES struct, core of the emulator.
pub mod nes;
/// Contain some useful data structure.
pub mod utils;

mod controllers;
mod cartridge;
mod cpu;
mod ppu;
mod apu;
mod bus;

/// Configuration to pass to the emulator.
pub struct Config {
    pub palette_path: Option<String>,
    pub display_cpu_logs: bool,
}

impl Config {
    /// Create a new configuration for the NES emulator.
    /// the `palette_path` argument should lead to a valid .pal file.
    pub fn new(
        palette_path: Option<&str>,
        display_cpu_logs: bool,
    ) -> Self {
        Config {
            palette_path: palette_path.map(str::to_string),
            display_cpu_logs,
        }
    }

    /// Generate a default configuration for the NES emulator.
    pub fn default() -> Self {
        Config {
            palette_path: None,
            display_cpu_logs: false,
        }
    }
}