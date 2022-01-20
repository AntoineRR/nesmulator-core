// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses winit and pixels crate

// ===== IMPORTS =====

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::ppu::palette::ARGBColor;

// ===== CONSTANTS =====

pub const MAIN_WINDOW_WIDTH: u32 = 256;
pub const MAIN_WINDOW_HEIGHT: u32 = 240;

pub const DEBUG_WINDOW_WIDTH: u32 = 256;
pub const DEBUG_WINDOW_HEIGHT: u32 = 128 + 2 + 6; // 2 rows to separate pattern tables and palette

// ===== STRUCT =====

#[derive(Debug)]
pub struct GUI {
    // Windows
    main_window: Window,
    // Screen buffers
    pub main_pixels: Pixels,
    // Debug
    pub debug: bool,
}

impl GUI {
    pub fn new(main_event_loop: &EventLoop<()>) -> Self {
        let window_size = LogicalSize::new(MAIN_WINDOW_WIDTH * 2, MAIN_WINDOW_HEIGHT * 2);
        let buffer_size = LogicalSize::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT);
        let main_window = WindowBuilder::new()
            .with_title("Nesmulator")
            .with_inner_size(window_size)
            .with_min_inner_size(buffer_size)
            .build(main_event_loop)
            .expect("Cannot create main window");

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &main_window);
        let main_pixels = Pixels::new(buffer_size.width, buffer_size.height, surface_texture).unwrap();

        GUI {
            main_window,
            main_pixels,
            debug: false,
        }
    }

    // Debugging window creation method
    pub fn toggle_debugging(&mut self) {
        if self.debug {
            let width = MAIN_WINDOW_WIDTH;
            let height = MAIN_WINDOW_HEIGHT;
            self.main_pixels.resize_buffer(width, height);
            self.debug = false;
        } else {
            let width = MAIN_WINDOW_WIDTH.max(DEBUG_WINDOW_WIDTH);
            let height = MAIN_WINDOW_HEIGHT + DEBUG_WINDOW_HEIGHT;
            self.main_pixels.resize_buffer(width, height);
            self.debug = true;
        }
    }

    // Updates the main screen buffer
    pub fn update_main_buffer(&mut self, index: usize, color: ARGBColor) {
        let pixel = &mut self.main_pixels.get_frame()[index * 4..index * 4 + 4];
        pixel[0] = color.red;
        pixel[1] = color.green;
        pixel[2] = color.blue;
        pixel[3] = color.alpha;
    }

    // Updates the debug screen buffer
    pub fn update_debug_buffer(&mut self, index: usize, color: ARGBColor) {
        let offset = index + (MAIN_WINDOW_WIDTH * MAIN_WINDOW_HEIGHT) as usize;
        let pixel = &mut self.main_pixels.get_frame()[offset * 4..offset * 4 + 4];
        pixel[0] = color.red;
        pixel[1] = color.green;
        pixel[2] = color.blue;
        pixel[3] = color.alpha;
    }

    // Updates what is displayed on the screen
    pub fn update(&mut self) {
        self.main_window.request_redraw();
    }
}
