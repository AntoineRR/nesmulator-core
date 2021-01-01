// Represents the NES system

// ===== IMPORTS =====

use cartridge::mapper::Mapper;

use crate::{CARTRIDGE, cartridge, cpu::cpu::CPU};
use crate::bus::Bus;

// ===== NES STRUCT =====

#[derive(Debug)]
pub struct NES {
    pub bus: Bus,
    pub cpu: CPU
}

impl NES {
    pub const fn new() -> Self {
        NES {
            bus: Bus::new(),
            cpu: CPU::new(),
        }
    }

    // Simulates the insertion of a NES cartridge
    // Sets the mapper that is needed to read the data of the cartridge
    pub fn insert_cartdrige(&mut self) {
        let mut mapper_number: u8 = 0;
        unsafe {
            mapper_number += CARTRIDGE.as_ref().unwrap().header.control_1 >> 4;
            mapper_number += (CARTRIDGE.as_ref().unwrap().header.control_2 >> 4) << 4;
        }
        
        self.bus.mapper = Mapper::new(mapper_number);
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self) {
        self.cpu.reset();
        let mut counter: u32 = 100;
        while counter != 0 {
            self.cpu.clock();
            println!("{:?}",self.cpu);
            counter -= 1;
        }
        println!("DONE.");
    }
}