use std::process::exit;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use clap::{App, Arg};
use log::error;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

use nes_emulator::controllers::ControllerInput;
use nes_emulator::gui::GUI;
use nes_emulator::nes::{Message, NES};
use nes_emulator::Config;

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
    let config = Config::new(palette_path, display_cpu_logs, debug_level);

    // Path to the game to launch
    let rom_path = matches.value_of("game").unwrap();

    // Create the GUI for displaying the graphics
    let event_loop = EventLoop::new();
    let gui = GUI::new(&event_loop);

    // Instantiate a NES and runs the game
    let mut nes = NES::new(gui, config);
    if let Err(e) = nes.insert_cartdrige(rom_path) {
        error!("Error parsing ROM: {}", e);
        exit(1);
    }

    // Spawn a thread to run the NES ROM and give it a channel receiver to handle events from the main loop
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    thread::spawn(move || nes.launch_game(rx));

    // Run the event loop
    let mut input_helper = WinitInputHelper::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::RedrawRequested(_) = event {
            tx.send(Message::DrawFrame).unwrap();
        }

        if input_helper.update(&event) {
            // Close event
            if input_helper.key_pressed(VirtualKeyCode::Escape) || input_helper.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize event
            if let Some(size) = input_helper.window_resized() {
                tx.send(Message::ResizeWindow(size.width, size.height))
                    .unwrap();
            }
            // Debug window
            if input_helper.key_pressed(VirtualKeyCode::E) {
                tx.send(Message::ToggleDebugWindow).unwrap();
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
            tx.send(Message::Input((0, input))).unwrap();
        }
    });
}
