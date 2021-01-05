mod cpu;
mod bus;
mod cartridge;
mod ppu;
mod nes;
mod gui;

use std::{path::Path, sync::{Arc, Mutex}};

use bus::Bus;
use cartridge::cartridge::Cartridge;
use cpu::cpu::CPU;
use nes::NES;
use ppu::ppu::PPU;
use gui::GUI;

fn main() {
    println!("NES Emulator");

    let path: &Path = Path::new("../ROM/Tests/nestest.nes");
    let cartridge: Cartridge = Cartridge::new(path);

    let p_gui: Arc<Mutex<GUI>> = Arc::new(Mutex::new(GUI::new()));

    let p_ppu: Arc<Mutex<PPU>> = Arc::new(Mutex::new(PPU::new(p_gui.clone())));
    let p_bus: Arc<Mutex<Bus>> = Arc::new(Mutex::new(Bus::new(p_ppu.clone())));
    let p_cpu: Arc<Mutex<CPU>> = Arc::new(Mutex::new(CPU::new(p_bus.clone())));
    let mut nes: NES = NES::new(p_bus.clone(), p_cpu, p_ppu.clone());

    nes.insert_cartdrige(cartridge);
    nes.launch_game();
}
