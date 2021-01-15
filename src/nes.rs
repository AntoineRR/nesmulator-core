// Represents the NES system

// ===== IMPORTS =====

use std::{sync::{Arc, Mutex}, time::Instant};

use cartridge::{cartridge::Cartridge, mapper::Mapper};

use crate::{cartridge, cpu::{cpu::CPU, enums::Interrupt}, gui::GUI};
use crate::bus::Bus;
use crate::ppu::ppu::PPU;

// ===== NES STRUCT =====

#[derive(Debug)]
pub struct NES {
    pub p_bus: Arc<Mutex<Bus>>,
    pub p_cpu: Arc<Mutex<CPU>>,
    pub p_ppu: Arc<Mutex<PPU>>,
    pub p_gui: Arc<Mutex<GUI>>
}

impl NES {
    pub fn new(p_bus: Arc<Mutex<Bus>>, p_cpu: Arc<Mutex<CPU>>, p_ppu: Arc<Mutex<PPU>>, p_gui: Arc<Mutex<GUI>>) -> Self {
        NES {
            p_bus,
            p_cpu,
            p_ppu,
            p_gui
        }
    }

    // Simulates the insertion of a NES cartridge
    // Sets the mapper that is needed to read the data of the cartridge
    pub fn insert_cartdrige(&mut self, cartridge: Cartridge) {
        let o_p_mapper: Option<Arc<Mutex<Mapper>>> = Some(Arc::new(Mutex::new(Mapper::new(cartridge))));
        self.p_bus.lock().unwrap().o_p_mapper = o_p_mapper.clone();
        self.p_ppu.lock().unwrap().ppu_bus.o_p_mapper = o_p_mapper.clone();
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self) {
        
        self.p_cpu.lock().unwrap().reset();
        let mut counter: u32 = 0;
        //self.p_cpu.lock().unwrap().pc = 0xC000; // Run nestest in automation mode (Fails at C6BD because of unofficial opcode)
        loop {
            //let now = Instant::now();

            // CPU is clocked every 3 PPU cycles
            if counter%3 == 0 {
                self.p_cpu.lock().unwrap().clock();
            }

            // Clock PPU
            self.p_ppu.lock().unwrap().clock();

            // Check if an NMI interrupt should be thrown
            if self.p_ppu.lock().unwrap().emit_nmi {
                self.p_ppu.lock().unwrap().emit_nmi = false;
                self.p_cpu.lock().unwrap().interrupt(Interrupt::NMI);
            }

            // Check if a key is pressed
            self.p_gui.lock().unwrap().check_keys(self.p_cpu.clone());

            // if self.p_gui.lock().unwrap().frame_ready {
            //     println!("CPU : {}, PPU : {}",self.p_cpu.lock().unwrap().total_clock, self.p_ppu.lock().unwrap().total_clock);
            //     self.p_cpu.lock().unwrap().total_clock = 0;
            //     self.p_ppu.lock().unwrap().total_clock = 0;
            //     self.p_gui.lock().unwrap().frame_ready = false;
            // }

            counter += 1;
            //println!("{}",now.elapsed().as_nanos());
        }
    }
}