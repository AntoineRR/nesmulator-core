// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses minifb

// ===== IMPORTS =====

use std::{env::consts::FAMILY, sync::{Arc, Mutex}};

use minifb::{Key, KeyRepeat, ScaleMode, Window, WindowOptions};

use crate::{cpu::cpu::CPU, ppu::palette::ARGBColor};

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
    // Keys
    pub lock_keys: bool,
    // Debug
    pub debug: bool,
    pub frame_ready: bool
}

impl GUI {
    pub fn new() -> Self {
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

        GUI {
            main_window: Arc::new(Mutex::new(main_window)),
            debugging_window: Arc::new(Mutex::new(None)),
            main_buffer: [0;MAIN_WINDOW_WIDTH*MAIN_WINDOW_HEIGHT],
            debug_buffer: [0;DEBUG_WINDOW_WIDTH*DEBUG_WINDOW_HEIGHT],
            lock_keys: false,
            debug: false,
            frame_ready: false
        }
    }

    // Debugging window creation method
    pub fn create_debugging_window(&mut self) {
        let debugging_window = Some(Window::new(
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
        self.debugging_window = Arc::new(Mutex::new(debugging_window));
    }

    // Debugging window destruction method
    pub fn destroy_debugging_window(&mut self) {
        
    }

    // Updates the main screen buffer
    pub fn update_main_buffer(&mut self, index: u32, color: ARGBColor) {
        self.main_buffer[index as usize] = self.convert_color(color);
    }

    // Updates the debug screen buffer
    pub fn update_debug_buffer(&mut self, index: u32, color:ARGBColor) {
        self.debug_buffer[index as usize] = self.convert_color(color);
    }

    // Updates what is displayed on the screen
    pub fn update(&mut self) {
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
        self.frame_ready = true;
    }

    // Converts the ARGB_Color struct used in the NES emulator
    // to a format usable by the GUI library (u32 for minifb)
    pub fn convert_color(&self, color: ARGBColor) -> u32 {
        (color.alpha as u32) << 24 ^ (color.red as u32) << 16 ^ (color.green as u32) << 8 ^ color.blue as u32
    }

    // ===== INPUTS =====

    pub fn check_keys(&mut self, p_cpu: Arc<Mutex<CPU>>) {
        if !self.lock_keys {
            if self.main_window.lock().unwrap().is_key_down(Key::R) {
                self.reset_cpu(p_cpu);
                self.lock_keys = true;
            }
            else if self.main_window.lock().unwrap().is_key_pressed(Key::E, KeyRepeat::No) {
                self.toggle_debugging();
                self.lock_keys = true;
            }
            else {
                self.lock_keys = false;
            }
        }
    }

    // r => Reset CPU
    pub fn reset_cpu(&self, p_cpu: Arc<Mutex<CPU>>) {
        p_cpu.lock().unwrap().reset();
    }

    // e => Toggle the display of the debugging window
    pub fn toggle_debugging(&mut self) {
        if self.debug {
            if self.debugging_window.lock().unwrap().is_some() {
                self.destroy_debugging_window();
                self.debug = false;
            }
        }
        else {
            if self.debugging_window.lock().unwrap().is_none() {
                self.create_debugging_window();
                self.debug = true;
            }
        }
    }
}