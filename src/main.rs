mod cpu;
mod bus;
mod cartridge;
mod nes;

use std::path::Path;

use cartridge::cartridge::Cartridge;
use nes::NES;

static mut NES: NES = NES::new();
static mut CARTRIDGE: Option<Cartridge> = None;

fn main() {
    println!("NES Emulator");

    let path: &Path = Path::new("../ROM/Tests/nestest.nes");
    
    unsafe {
        CARTRIDGE = Some(Cartridge::new(path));
    }

    unsafe {
        NES.insert_cartdrige();
        NES.launch_game();
    }
}
