use env_logger::Env;
use log::warn;

pub mod controllers;
pub mod gui;
pub mod nes;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod ppu;

const DEFAULT_DEBUG_LEVEL: &str = "info";

pub struct Config {
    palette_path: Option<String>,
    display_cpu_logs: bool,
}

impl Config {
    pub fn new(
        palette_path: Option<&str>,
        display_cpu_logs: bool,
        debug_level: Option<&str>,
    ) -> Self {
        let debug_level = if let Some(value) = debug_level {
            match value {
                "0" => "error",
                "1" => "warn",
                "2" => "info",
                "3" => "debug",
                "4" => "trace",
                d => {
                    warn!("Invalid debug level : {:?}, value must be in [0;4]. Using default debug level.", d);
                    DEFAULT_DEBUG_LEVEL
                }
            }
        } else {
            DEFAULT_DEBUG_LEVEL
        };

        // Setup logger
        // Logs level from winit and pixels crates are set to warn
        env_logger::Builder::from_env(Env::default().default_filter_or(
            debug_level.to_owned()
                + ",gfx_memory=warn,gfx_backend_vulkan=warn,gfx_descriptor=warn,winit=warn,mio=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn",
        ))
        .init();

        Config {
            palette_path: palette_path.map(str::to_string),
            display_cpu_logs,
        }
    }
}
