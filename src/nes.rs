// Represents the NES system

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use cartridge::{cartridge::Cartridge, mapper::Mapper};

use crate::{cartridge, cpu::cpu::CPU};
use crate::bus::Bus;
use crate::ppu::ppu::PPU;

// ===== NES STRUCT =====

#[derive(Debug)]
pub struct NES {
    pub p_bus: Arc<Mutex<Bus>>,
    pub p_cpu: Arc<Mutex<CPU>>,
    pub p_ppu: Arc<Mutex<PPU>>
}

impl NES {
    pub fn new(p_bus: Arc<Mutex<Bus>>, p_cpu: Arc<Mutex<CPU>>, p_ppu: Arc<Mutex<PPU>>) -> Self {
        NES {
            p_bus,
            p_cpu,
            p_ppu
        }
    }

    // Simulates the insertion of a NES cartridge
    // Sets the mapper that is needed to read the data of the cartridge
    pub fn insert_cartdrige(&mut self, cartridge: Cartridge) {
        self.p_bus.lock().unwrap().mapper = Arc::new(Mutex::new(Some(Mapper::new(cartridge))));
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self) {
        
        self.p_cpu.lock().unwrap().reset();
        let mut counter: u32 = 0;
        
        loop {
            // Calculate one pixel color
            self.p_ppu.lock().unwrap().clock();
            // CPU is clocked every 3 PPU cycles
            if counter%3 == 0 {
                self.p_cpu.lock().unwrap().clock();
            }
            counter += 1;
        }
    }
}