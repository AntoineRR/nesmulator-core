mod cpu;
mod bus;
mod cartridge;
mod ppu;
mod nes;
mod gui;
mod controllers;

use std::{path::Path, sync::{Arc, Mutex}};

use bus::Bus;
use cartridge::cartridge::Cartridge;
use cpu::cpu::CPU;
use nes::NES;
use ppu::ppu::PPU;
use gui::GUI;

fn main() {
    // Create the cartridge based on the file located at the given path
    //let path: &Path = Path::new("../ROM/Tests/nestest.nes");
    let path: &Path = Path::new("../ROM/smb.nes");
    let cartridge: Cartridge = Cartridge::new(path);

    // Create the GUI for displaying the graphics
    let p_gui: Arc<Mutex<GUI>> = Arc::new(Mutex::new(GUI::new()));

    // Creates the NES architecture
    let p_ppu: Arc<Mutex<PPU>> = Arc::new(Mutex::new(PPU::new(p_gui.clone())));
    let p_bus: Arc<Mutex<Bus>> = Arc::new(Mutex::new(Bus::new(p_ppu.clone())));
    let p_cpu: Arc<Mutex<CPU>> = Arc::new(Mutex::new(CPU::new(p_bus.clone())));
    let mut nes: NES = NES::new(p_bus.clone(), p_cpu.clone(), p_ppu.clone(), p_gui.clone());

    // Runs the game on the cartridge
    nes.insert_cartdrige(cartridge);
    nes.launch_game();
}
