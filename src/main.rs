mod cpu;
mod bus;
mod cartridge;
mod ppu;
mod nes;
mod gui;
mod controllers;

use std::{path::Path, sync::{Arc, Mutex}, thread};

use bus::Bus;
use cartridge::cartridge::Cartridge;
use controllers::ControllerInput;
use cpu::cpu::CPU;
use nes::NES;
use ppu::ppu::PPU;
use gui::GUI;
use winit::{event::{Event, VirtualKeyCode}, event_loop::{ControlFlow, EventLoop}};
use winit_input_helper::WinitInputHelper;

fn main() {
    // Create the cartridge based on the file located at the given path
    
    // CPU TESTS ROMS

    // let path: &Path = Path::new("../ROM/Tests/nestest.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/branch_timing_tests/1.Branch_Basics.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/branch_timing_tests/2.Backward_Branch.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/branch_timing_tests/3.Forward_Branch.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/cpu_timing_test6/cpu_timing_test.nes");
    
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/01-implied.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/02-immediate.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/03-zero_page.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/04-zp_xy.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/05-absolute.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/06-abs_xy.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/07-ind_x.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/08-ind_y.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/09-branches.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/10-stack.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/nes_instr_test/rom_singles/11-special.nes");

    // PPU TEST ROMS

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/blargg_ppu_tests_2005.09.15b/palette_ram.nes");
    //let path: &Path = Path::new("../ROM/Tests/nes-test-roms/blargg_ppu_tests_2005.09.15b/sprite_ram.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/blargg_ppu_tests_2005.09.15b/vram_access.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/scanline/scanline.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/ppu_open_bus/ppu_open_bus.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/1.frame_basics.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/2.vbl_timing.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/3.even_odd_frames.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/4.vbl_clear_timing.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/5.nmi_suppression.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/6.nmi_disable.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/vbl_nmi_timing/7.nmi_timing.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/oam_read/oam_read.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/oam_stress/oam_stress.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/04.flip.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/05.left_clip.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/06.right_edge.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/08.double_height.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/09.timing_basics.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/10.timing_order.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_hit_tests_2005.10.05/11.edge_timing.nes");

    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_overflow_tests/1.Basics.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_overflow_tests/2.Details.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_overflow_tests/3.Timing.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_overflow_tests/4.Obscure.nes");
    // let path: &Path = Path::new("../ROM/Tests/nes-test-roms/sprite_overflow_tests/5.Emulator.nes");

    // GAMES

    let path: &Path = Path::new("../ROM/zelda.nes");

    let cartridge: Cartridge = Cartridge::new(path);

    // Create the Eventloop for interacting with the window
    let event_loop = EventLoop::new();
    // Create the GUI for displaying the graphics
    let p_gui: Arc<Mutex<GUI>> = Arc::new(Mutex::new(GUI::new(&event_loop)));

    // Creates the NES architecture
    let p_ppu: Arc<Mutex<PPU>> = Arc::new(Mutex::new(PPU::new(p_gui.clone())));
    let p_bus: Arc<Mutex<Bus>> = Arc::new(Mutex::new(Bus::new(p_ppu.clone())));
    let p_cpu: Arc<Mutex<CPU>> = Arc::new(Mutex::new(CPU::new(p_bus.clone())));
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
            if main_pixels.lock().unwrap()
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
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
