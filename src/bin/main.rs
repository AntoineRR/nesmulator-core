use std::process::exit;

use nesmulator_core::nes::NES;

fn main() {
    // Path to the game to launch
    let rom_path = "../ROM/rom.nes";

    // Instantiate a NES and connect a ROM file
    let mut nes = NES::new();
    if let Err(e) = nes.insert_cartdrige(rom_path) {
        println!("Error parsing ROM: {e}");
        exit(1);
    }

    // Main emulation loop
    // This only clocks the NES as fast as possible.
    // For an example of a full GUI using this crate, check out https://github.com/AntoineRR/nesmulator-gui
    loop {
        nes.clock();
    }
}
