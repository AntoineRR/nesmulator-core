// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses minifb

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use minifb::{ScaleMode, Window, WindowOptions};

use crate::ppu::palette::ARGBColor;

// ===== CONSTANTS =====

pub const WINDOW_WIDTH: usize = 256;
pub const WINDOW_HEIGHT: usize = 240;

// ===== STRUCT =====

#[derive(Debug)]
pub struct GUI {
    window: Arc<Mutex<Window>>,
    pub buffer: [u32;256*240]
}

impl GUI {
    pub fn new() -> Self {
        let window = Window::new(
            "Nesmulator",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to open Window");

        GUI {
            window: Arc::new(Mutex::new(window)),
            buffer: [0;256*240]
        }
    }

    // Updates the screen buffer
    pub fn update_buffer(&mut self, index: u32, color: ARGBColor) {
        self.buffer[index as usize] = self.convert_color(color);
    }

    // Updates what is displayed on the screen
    pub fn update(&self) {
        self.window
            .lock()
            .unwrap()
            .update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }

    // Converts the ARGB_Color struct used in the NES emulator
    // to a format usable by the GUI library (u32 for minifb)
    pub fn convert_color(&self, color: ARGBColor) -> u32 {
        (color.alpha as u32) << 24 ^ (color.red as u32) << 16 ^ (color.green as u32) << 8 ^ color.blue as u32
    }
}