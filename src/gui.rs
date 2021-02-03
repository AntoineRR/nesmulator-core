// Contains all the methods that are necessary for the GUI
// As the emulator just call those methods, it should be easy
// to change the GUI library if needed.
// Currently, it uses winit and pixels crate

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use pixels::{Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::{Window, WindowBuilder}};

use crate::{bus::Bus, ppu::palette::ARGBColor};

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
    debugging_window: Arc<Mutex<Option<Window>>>,
    // Keys pressed
    pub inputs: Arc<Mutex<u8>>,
    // Screen buffers
    pub main_pixels: Arc<Mutex<Pixels<Window>>>,
    pub debug_pixels: Option<Pixels<Window>>,
    // Debug
    pub debug: bool,
    pub frame_ready: bool
}

impl GUI {
    pub fn new(main_event_loop: &EventLoop<()>) -> Self {
        let main_window = WindowBuilder::new()
            .with_title("Nesmulator")
            .with_inner_size(LogicalSize::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT))
            .build(main_event_loop)
            .expect("Cannot create main window");

        let surface_texture = SurfaceTexture::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, &main_window);
        let main_pixels = Arc::new(Mutex::new(
            Pixels::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, surface_texture).unwrap()));

        let inputs = Arc::new(Mutex::new(0));
        
        GUI {
            main_window,
            debugging_window: Arc::new(Mutex::new(None)),
            inputs: inputs.clone(),
            main_pixels,
            debug_pixels: None,
            debug: false,
            frame_ready: false
        }
    }

    // Debugging window creation method
    pub fn create_debugging_window(&mut self, debug_event_loop: &EventLoop<()>) {
        let debugging_window = WindowBuilder::new()
            .with_title("Nesmulator")
            .with_inner_size(LogicalSize::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT))
            .build(debug_event_loop)
            .expect("Cannot create debug window");
        
        let surface_texture 
            = SurfaceTexture::new(DEBUG_WINDOW_WIDTH, DEBUG_WINDOW_HEIGHT, &debugging_window);
        self.debug_pixels = Some(Pixels::new(DEBUG_WINDOW_WIDTH, DEBUG_WINDOW_HEIGHT, surface_texture).unwrap());
        
        self.debugging_window = Arc::new(Mutex::new(Some(debugging_window)));
    }

    // Updates the main screen buffer
    pub fn update_main_buffer(&mut self, index: usize, color: ARGBColor) {
        let mut main_pixels = self.main_pixels.lock().unwrap();
        let pixel = &mut main_pixels.get_frame()[index*4..index*4+4];
        pixel[0] = color.red;
        pixel[1] = color.green;
        pixel[2] = color.blue;
        pixel[3] = color.alpha;
    }

    // Updates the debug screen buffer
    pub fn update_debug_buffer(&mut self, index: usize, color:ARGBColor) {
        let pixel = &mut self.debug_pixels.as_mut().unwrap().get_frame()[index*4..index*4+4];
        pixel[0] = color.red;
        pixel[1] = color.green;
        pixel[2] = color.blue;
        pixel[3] = color.alpha;
    }

    // Updates what is displayed on the screen
    pub fn update(&mut self) {
        self.main_window.request_redraw();
        self.frame_ready = true;
    }

    // ===== INPUTS =====

    pub fn check_keys(&mut self, p_bus: Arc<Mutex<Bus>>) {
        p_bus.lock().unwrap().controllers[0].buffer = *self.inputs.lock().unwrap();
        *self.inputs.lock().unwrap() = 0;
    }
}