use std::error::Error;
use std::process::exit;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use clap::{App, Arg};
use env_logger::Env;
use log::{error, info, warn};
use pixels::{Pixels, SurfaceTexture};
use sdl2::audio::AudioSpecDesired;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

use nesmulator_core::nes::NES;
use nesmulator_core::utils::ARGBColor;
use nesmulator_core::utils::ControllerInput;
use nesmulator_core::Config;

// Different messages that can be thrown at the NES by the event loop
#[derive(PartialEq)]
enum Message {
    Input(usize, u8),
    Reset,
    DrawFrame,
    ChangePaletteId(u8),
    ResizeWindow(u32, u32),
    ToggleDebugWindow,
    CloseApp,
}

const DEFAULT_DEBUG_LEVEL: &str = "info";
const MIN_AUDIO_QUEUE_SIZE: u32 = 4 * 4410;

fn main() {
    // CLI creation
    let matches = App::new("Nesmulator")
        .version("0.1.0")
        .author("AntoineRR <ant.romero2@orange.fr>")
        .about("A simple NES emulator written in Rust")
        .arg(
            Arg::new("game")
                .index(1)
                .value_name("FILE")
                .about("Sets the nes file to run in the emulator")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .value_name("INT")
                .takes_value(true)
                .about("Turn debugging information on"),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .about("Display the CPU logs to the console"),
        )
        .arg(
            Arg::new("palette")
                .short('p')
                .long("palette")
                .value_name("FILE")
                .takes_value(true)
                .about("Sets a palette from a .pal file"),
        )
        .get_matches();

    // Get all configuration informations
    let palette_path = matches.value_of("palette");
    let display_cpu_logs = matches.is_present("log");
    let debug_level = matches.value_of("debug");
    let config = Config::new(palette_path, display_cpu_logs);

    init_env_logger(debug_level);

    // Path to the game to launch
    let rom_path = matches.value_of("game").unwrap();

    // Create the GUI for displaying the graphics
    let event_loop = EventLoop::new();
    let mut gui = GUI::new(&event_loop);

    // Instantiate a NES and connect a ROM file
    let mut nes = NES::from_config(config);
    if let Err(e) = nes.insert_cartdrige(rom_path) {
        error!("Error parsing ROM: {}", e);
        exit(1);
    }
    info!("ROM {} successfully loaded.", rom_path);

    // Load a save for the current cartridge, if any
    if let Ok(_) = nes.load_save() {
        info!("Save successfully loaded.");
    }

    // Spawn a thread to run the NES ROM and give it a channel receiver to handle events from the main loop
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    thread::spawn(move || run_nes(&mut nes, &mut gui, rx));

    // Run the event loop
    let mut palette_id = 0;
    let mut input_helper = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            send_message(&tx, Message::DrawFrame, control_flow);
        }

        if input_helper.update(&event) {
            // Close event
            if input_helper.key_pressed(VirtualKeyCode::Escape) || input_helper.quit() {
                *control_flow = ControlFlow::Exit;
                send_message(&tx, Message::CloseApp, control_flow);
                info!("Closing application...");
                exit(0);
            }
            // Resize event
            if let Some(size) = input_helper.window_resized() {
                send_message(
                    &tx,
                    Message::ResizeWindow(size.width, size.height),
                    control_flow,
                );
            }
            // Debug window
            if input_helper.key_pressed(VirtualKeyCode::E) {
                send_message(&tx, Message::ToggleDebugWindow, control_flow);
            }
            // Reset
            if input_helper.key_pressed(VirtualKeyCode::R) {
                send_message(&tx, Message::Reset, control_flow);
            }
            // Change debug palette
            if input_helper.key_pressed(VirtualKeyCode::Left) {
                if palette_id == 0 {
                    palette_id = 7;
                } else {
                    palette_id -= 1;
                }
                send_message(&tx, Message::ChangePaletteId(palette_id), control_flow);
            }
            if input_helper.key_pressed(VirtualKeyCode::Right) {
                if palette_id == 7 {
                    palette_id = 0;
                } else {
                    palette_id += 1;
                }
                send_message(&tx, Message::ChangePaletteId(palette_id), control_flow);
            }
            // Controller inputs
            let mut input = 0;
            if input_helper.key_held(VirtualKeyCode::Z) {
                input |= ControllerInput::Up as u8;
            }
            if input_helper.key_held(VirtualKeyCode::Q) {
                input |= ControllerInput::Left as u8;
            }
            if input_helper.key_held(VirtualKeyCode::S) {
                input |= ControllerInput::Down as u8;
            }
            if input_helper.key_held(VirtualKeyCode::D) {
                input |= ControllerInput::Right as u8;
            }
            if input_helper.key_held(VirtualKeyCode::X) {
                input |= ControllerInput::Start as u8;
            }
            if input_helper.key_held(VirtualKeyCode::C) {
                input |= ControllerInput::Select as u8;
            }
            if input_helper.key_held(VirtualKeyCode::I) {
                input |= ControllerInput::A as u8;
            }
            if input_helper.key_held(VirtualKeyCode::O) {
                input |= ControllerInput::B as u8;
            }
            send_message(&tx, Message::Input(0, input), control_flow);
        }
    });
}

fn send_message(tx: &Sender<Message>, message: Message, control_flow: &mut ControlFlow) {
    if let Err(_) = tx.send(message) {
        error!("Receiving thread 'run_nes' panicked");
        *control_flow = ControlFlow::Exit;
        exit(1);
    }
}

fn init_env_logger(debug_level: Option<&str>) {
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
}

fn run_nes(nes: &mut NES, gui: &mut GUI, rx: Receiver<Message>) {
    info!("Running NES emulation...");

    // Sound
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_audio_specs = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024),
    };

    let queue = audio_subsystem
        .open_queue(None, &desired_audio_specs)
        .unwrap();
    queue.resume();

    let target_time = nes.get_one_frame_duration();
    let mut time = Instant::now();

    loop {
        // Run one clock of emulation
        nes.clock();

        // Handle message from the main thread
        if let Ok(m) = rx.try_recv() {
            let keep_running = handle_message(nes, gui, m);
            if !keep_running {
                break;
            }
        }

        // Render frame if ready
        if let Some(frame) = nes.get_frame_buffer() {
            gui.update_main_buffer(&frame);
            if gui.debug {
                gui.debug(
                    &nes.get_pattern_table(0).unwrap(),
                    &nes.get_pattern_table(1).unwrap(),
                    &nes.get_palette().unwrap(),
                );
            }
            gui.render().unwrap();

            // Synchronize with sound
            if !nes.is_producing_samples() && queue.size() < MIN_AUDIO_QUEUE_SIZE {
                nes.produce_samples(true);
            } else if nes.is_producing_samples() && queue.size() > MIN_AUDIO_QUEUE_SIZE {
                nes.produce_samples(false);
            }
            queue.queue(&nes.get_samples()[..]);

            // Synchronize the emulation to run at the correct speed
            let elapsed_time = time.elapsed();
            if elapsed_time < target_time {
                spin_sleep::sleep(target_time - elapsed_time);
            }
            time = Instant::now();
        }
    }
}

fn handle_message(nes: &mut NES, gui: &mut GUI, message: Message) -> bool {
    match message {
        Message::Input(id, input) => {
            if let Err(e) = nes.input(id, input) {
                error!("Failed to handle controller input: {}", e);
                exit(1);
            }
        }
        Message::Reset => nes.reset(),
        Message::ResizeWindow(width, height) => gui.resize(width, height),
        Message::DrawFrame => gui.redraw(),
        Message::ChangePaletteId(id) => nes.set_debug_palette_id(id).unwrap(),
        Message::ToggleDebugWindow => gui.toggle_debugging(),
        Message::CloseApp => {
            if let Ok(_) = nes.save() {
                info!("Game successfully saved.");
            }
            return false;
        }
    }
    return true;
}

// All the methods for the GUI

const MAIN_WINDOW_WIDTH: u32 = 256;
pub const MAIN_WINDOW_HEIGHT: u32 = 240;

pub const DEBUG_WINDOW_WIDTH: u32 = 256;
pub const DEBUG_WINDOW_HEIGHT: u32 = 240 + 2 + 128 + 2 + 6; // From top to bottom: main window | pattern table | palette

#[derive(Debug)]
pub struct GUI {
    main_window: Window,
    main_pixels: Pixels,
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
        let main_pixels =
            Pixels::new(buffer_size.width, buffer_size.height, surface_texture).unwrap();

        GUI {
            main_window,
            main_pixels,
            debug: false,
        }
    }

    pub fn toggle_debugging(&mut self) {
        if self.debug {
            let width = MAIN_WINDOW_WIDTH;
            let height = MAIN_WINDOW_HEIGHT;
            self.main_pixels.resize_buffer(width, height);
            self.debug = false;
        } else {
            let width = DEBUG_WINDOW_WIDTH;
            let height = DEBUG_WINDOW_HEIGHT;
            self.main_pixels.resize_buffer(width, height);
            self.debug = true;
        }
    }

    fn add_pattern_tables(
        &mut self,
        buffer: &mut [ARGBColor],
        pattern_table_0: &[ARGBColor],
        pattern_table_1: &[ARGBColor],
    ) {
        for (i, color) in pattern_table_0.iter().enumerate() {
            buffer[(i / 128) * 256 + i % 128] = *color;
        }
        for (i, color) in pattern_table_1.iter().enumerate() {
            buffer[(i / 128) * 256 + i % 128 + 128] = *color;
        }
    }

    fn add_separation(&mut self, buffer: &mut [ARGBColor]) {
        for i in 0..512 {
            buffer[i] = ARGBColor::light_gray();
        }
    }

    fn add_palette(&mut self, buffer: &mut [ARGBColor], palette: &[ARGBColor]) {
        for (offset, color) in palette.iter().enumerate() {
            for i in 0..6 {
                for j in 0..6 {
                    let index = (offset * 6) + (((offset % 4) == 0) as usize) * 2 + i + j * 256;
                    buffer[index] = *color;
                }
            }
        }
    }

    pub fn debug(
        &mut self,
        pattern_table_0: &[ARGBColor],
        pattern_table_1: &[ARGBColor],
        palette: &[ARGBColor],
    ) {
        const BUFFER_SIZE: usize =
            ((DEBUG_WINDOW_HEIGHT - MAIN_WINDOW_HEIGHT) * DEBUG_WINDOW_WIDTH) as usize;
        let mut buffer = [ARGBColor::black(); BUFFER_SIZE];
        let mut offset = 0;
        self.add_separation(&mut buffer[offset..offset + 512]);
        offset += 512;
        self.add_pattern_tables(
            &mut buffer[offset..offset + 32768],
            pattern_table_0,
            pattern_table_1,
        );
        offset += 32768;
        self.add_separation(&mut buffer[offset..offset + 512]);
        offset += 512;
        self.add_palette(&mut buffer[offset..offset + 1536], palette);

        self.update_debug_buffer(&buffer);
    }

    pub fn update_main_buffer(&mut self, buffer: &[ARGBColor; 61_440]) {
        for (i, color) in buffer.iter().enumerate() {
            self.update_pixel(i, color);
        }
    }

    fn update_debug_buffer(&mut self, buffer: &[ARGBColor]) {
        let offset = (MAIN_WINDOW_WIDTH * MAIN_WINDOW_HEIGHT) as usize;
        for (i, color) in buffer.iter().enumerate() {
            self.update_pixel(offset + i, color);
        }
    }

    pub fn redraw(&mut self) {
        self.main_window.request_redraw();
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        self.main_pixels.render()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.main_pixels.resize_surface(width, height);
    }

    fn update_pixel(&mut self, offset: usize, color: &ARGBColor) {
        let pixel = &mut self.main_pixels.get_frame()[offset * 4..offset * 4 + 4];
        pixel[0] = color.red;
        pixel[1] = color.green;
        pixel[2] = color.blue;
        pixel[3] = color.alpha;
    }
}
