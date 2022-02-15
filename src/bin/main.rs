use std::process::exit;

use nesmulator_core::{nes::NES, Config};

const NESTEST_ROM_PATH: &str = "../ROM/Tests/nestest.nes";

fn main() {
    nestest_automation(false);
}

fn nestest_automation(run_once: bool) {
    // Path to the ROM to launch
    let rom_path = NESTEST_ROM_PATH;

    // Instantiate a NES and connect the nestest ROM file
    let mut nes = NES::from_config(Config {
        display_cpu_logs: false, // Change to true to follow each CPU instruction
        palette_path: None,
    });
    if let Err(e) = nes.insert_cartdrige(rom_path) {
        println!("Error parsing ROM: {e}");
        exit(1);
    }

    // Set CPU pc to 0xC000 to run nestest on automation
    nes.set_program_counter_at(0xC000);

    let mut cycle_count = 0;

    // Main emulation loop
    // This only clocks the NES as fast as possible.
    // For an example of a full GUI using this crate, check out https://github.com/AntoineRR/nesmulator-gui
    // The loop will stop at the end of the nestest ROM or start it again.
    loop {
        nes.clock();
        cycle_count += 1;
        if cycle_count % (26560 * 3) == 0 {
            if run_once {
                break;
            } else {
                nes.restart();
                nes.insert_cartdrige(NESTEST_ROM_PATH).unwrap();
                nes.set_program_counter_at(0xC000);
                cycle_count = 0;
            }
        }
    }
}
