mod bus;
mod cartridge;
mod controllers;
mod cpu;
mod gui;
mod nes;
mod ppu;

use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use bus::Bus;
use cartridge::cartridge::Cartridge;
use clap::{App, Arg};
use controllers::ControllerInput;
use cpu::cpu::CPU;
use env_logger::Env;
use gui::GUI;
use log::{error, warn};
use nes::NES;
use ppu::ppu::PPU;
use winit::{
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
};
use winit_input_helper::WinitInputHelper;

fn main() {
    // ===== APP CREATION AND ARGUMENT PARSING =====

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
                .value_name("LEVEL")
                .takes_value(true)
                .about("Turn debugging information on"),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .about("Display the CPU logs to the console"),
        )
        .get_matches();

    // Debug level

    let mut debug_level: &str = "warn";
    let mut is_debug_level_valid: bool = true;
    if let Some(value) = matches.value_of("debug") {
        match value {
            "0" => debug_level = "trace",
            "1" => debug_level = "debug",
            "2" => debug_level = "info",
            "3" => debug_level = "warn",
            "4" => debug_level = "error",
            _ => is_debug_level_valid = false,
        }
    }

    // Setup logger
    // Logs level from winit and pixels crates are set to warn
    env_logger::Builder::from_env(Env::default().default_filter_or(
        debug_level.to_owned()
            + ",gfx_memory=warn,gfx_backend_vulkan=warn,gfx_descriptor=warn,winit=warn,mio=warn",
    ))
    .init();

    if !is_debug_level_valid {
        warn!(
            "Invalid debug level : {:?}, value must be in [0;4]",
            matches.value_of("debug")
        );
    }

    // Display logs from cpu

    let display_cpu_logs: bool = matches.is_present("log");

    // Path to the game to launch

    let game = matches.value_of("game").unwrap();

    // ===== LAUNCH GAME =====

    let path: &Path = Path::new(game);

    let cartridge: Cartridge = Cartridge::new(path);

    // Create the Eventloop for interacting with the window
    let event_loop = EventLoop::new();
    // Create the GUI for displaying the graphics
    let p_gui = Arc::new(Mutex::new(GUI::new(&event_loop)));

    // Creates the NES architecture
    let p_ppu = Rc::new(RefCell::new(PPU::new(p_gui.clone())));
    let p_bus = Arc::new(Mutex::new(Bus::new(p_ppu.clone())));
    let p_cpu = Rc::new(RefCell::new(CPU::new(p_bus.clone(), display_cpu_logs)));
    let mut nes: NES = NES::new(p_bus.clone(), p_cpu.clone(), p_ppu.clone(), p_gui.clone());

    // Runs the game on the cartridge
    nes.insert_cartdrige(cartridge);
    thread::spawn(move || nes.launch_game());

    // Event loop for the window

    let mut input_helper = WinitInputHelper::new();
    let main_pixels = p_gui.clone().lock().unwrap().main_pixels.clone();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            if main_pixels
                .lock()
                .unwrap()
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input_helper.update(&event) {
            // Close event
            if input_helper.key_pressed(VirtualKeyCode::Escape) || input_helper.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize event
            if let Some(size) = input_helper.window_resized() {
                main_pixels.lock().unwrap().resize(size.width, size.height);
            }
            // Debug window
            if input_helper.key_pressed(VirtualKeyCode::E) {
                if !p_gui.lock().unwrap().debug {
                    //p_gui.lock().unwrap().create_debugging_window(&event_loop);
                    p_gui.lock().unwrap().debug = true;
                }
            }
            // Controller inputs
            p_bus.lock().unwrap().controllers[0].buffer = 0;
            if input_helper.key_held(VirtualKeyCode::Z) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Up as u8;
            }
            if input_helper.key_held(VirtualKeyCode::Q) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Left as u8;
            }
            if input_helper.key_held(VirtualKeyCode::S) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Down as u8;
            }
            if input_helper.key_held(VirtualKeyCode::D) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Right as u8;
            }
            if input_helper.key_held(VirtualKeyCode::X) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Start as u8;
            }
            if input_helper.key_held(VirtualKeyCode::C) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::Select as u8;
            }
            if input_helper.key_held(VirtualKeyCode::I) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::A as u8;
            }
            if input_helper.key_held(VirtualKeyCode::O) {
                p_bus.lock().unwrap().controllers[0].buffer |= ControllerInput::B as u8;
            }
        }
    });
}
