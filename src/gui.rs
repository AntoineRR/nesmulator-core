// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses minifb

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use minifb::{ScaleMode, Window, WindowOptions};

// ===== CONSTANTS =====

pub const WINDOW_WIDTH: usize = 256;
pub const WINDOW_HEIGHT: usize = 240;

// ===== STRUCT =====

#[derive(Debug)]
pub struct GUI {
    window: Arc<Mutex<Window>>
}

impl GUI {
    pub fn new() -> Self {
        let window = Window::new(
            "Nesmulator",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::Center,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to open Window");

        GUI {
            window: Arc::new(Mutex::new(window))
        }
    }

    pub fn update(&self, buffer: Vec<u32>) {
        self.window
            .lock()
            .unwrap()
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }
}