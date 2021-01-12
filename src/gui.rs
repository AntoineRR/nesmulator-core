// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses minifb

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use minifb::{ScaleMode, Window, WindowOptions};

use crate::ppu::palette::ARGBColor;

// ===== CONSTANTS =====

pub const MAIN_WINDOW_WIDTH: usize = 256;
pub const MAIN_WINDOW_HEIGHT: usize = 240;

pub const DEBUG_WINDOW_WIDTH: usize = 256;
pub const DEBUG_WINDOW_HEIGHT: usize = 128 + 2 + 6; // 2 rows to separate pattern tables and palette

// ===== STRUCT =====

#[derive(Debug)]
pub struct GUI {
    // Windows
    main_window: Arc<Mutex<Window>>,
    debugging_window: Arc<Mutex<Option<Window>>>,
    // Screen buffers
    pub main_buffer: [u32;MAIN_WINDOW_WIDTH*MAIN_WINDOW_HEIGHT],
    pub debug_buffer: [u32;DEBUG_WINDOW_WIDTH*DEBUG_WINDOW_HEIGHT],
    // Debug
    pub debug: bool
}

impl GUI {
    pub fn new(debug: bool) -> Self {
        let main_window = Window::new(
            "Nesmulator",
            MAIN_WINDOW_WIDTH,
            MAIN_WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to open Main Window");

        let debugging_window: Option<Window>;
        if debug {
            debugging_window = Some(Window::new(
                "Nesmulator - Debug",
                DEBUG_WINDOW_WIDTH,
                DEBUG_WINDOW_HEIGHT,
                WindowOptions {
                    resize: true,
                    scale_mode: ScaleMode::AspectRatioStretch,
                    ..WindowOptions::default()
                }
            )
            .expect("Unable to open Debug Window"));
        }
        else {
            debugging_window = None;
        }

        GUI {
            main_window: Arc::new(Mutex::new(main_window)),
            debugging_window: Arc::new(Mutex::new(debugging_window)),
            main_buffer: [0;MAIN_WINDOW_WIDTH*MAIN_WINDOW_HEIGHT],
            debug_buffer: [0;DEBUG_WINDOW_WIDTH*DEBUG_WINDOW_HEIGHT],
            debug
        }
    }

    // Updates the main screen buffer
    pub fn update_main_buffer(&mut self, index: u32, color: ARGBColor) {
        self.main_buffer[index as usize] = self.convert_color(color);
    }

    // Updates the debug screen buffer
    #[allow(dead_code)]
    pub fn update_debug_buffer(&mut self, index: u32, color:ARGBColor) {
        self.debug_buffer[index as usize] = self.convert_color(color);
    }

    // Updates what is displayed on the screen
    pub fn update(&self) {
        self.main_window
            .lock()
            .unwrap()
            .update_with_buffer(&self.main_buffer, MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)
            .unwrap();
        
        if self.debug {
            self.debugging_window
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .update_with_buffer(&self.debug_buffer, DEBUG_WINDOW_WIDTH, DEBUG_WINDOW_HEIGHT)
                .unwrap();
        }
    }

    // Converts the ARGB_Color struct used in the NES emulator
    // to a format usable by the GUI library (u32 for minifb)
    pub fn convert_color(&self, color: ARGBColor) -> u32 {
        (color.alpha as u32) << 24 ^ (color.red as u32) << 16 ^ (color.green as u32) << 8 ^ color.blue as u32
    }
}