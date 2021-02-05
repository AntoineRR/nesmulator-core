// Represents the NES system

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use cartridge::cartridge::Cartridge;

use crate::{cartridge, cpu::{cpu::CPU, enums::Interrupt}, gui::GUI};
use crate::bus::Bus;
use crate::ppu::ppu::PPU;

// ===== NES STRUCT =====

pub struct NES {
    // NES components
    pub p_bus: Arc<Mutex<Bus>>,
    pub p_cpu: Arc<Mutex<CPU>>,
    pub p_ppu: Arc<Mutex<PPU>>,
    pub p_gui: Arc<Mutex<GUI>>,

    // NES clock counter
    pub total_clock: u64,

    // DMA variables
    pub dma_started: bool,
    pub dma_hi_address: u8,
    pub dma_base_address: u8,
    pub dma_address: u8,
    pub dma_data: u8
}

unsafe impl Send for NES {}

impl NES {
    pub fn new(p_bus: Arc<Mutex<Bus>>, p_cpu: Arc<Mutex<CPU>>, p_ppu: Arc<Mutex<PPU>>, p_gui: Arc<Mutex<GUI>>) -> Self {
        NES {
            p_bus,
            p_cpu,
            p_ppu,
            p_gui,

            total_clock: 0,

            dma_started: false,
            dma_hi_address: 0,
            dma_base_address: 0,
            dma_address: 0,
            dma_data: 0
        }
    }

    // Simulates the insertion of a NES cartridge
    // Sets the mapper that is needed to read the data of the cartridge
    pub fn insert_cartdrige(&mut self, cartridge: Cartridge) {
        self.p_bus.lock().unwrap().o_p_mapper = Some(cartridge.mapper.clone());
        self.p_ppu.lock().unwrap().ppu_bus.o_p_mapper = Some(cartridge.mapper.clone());
    }

    // Resets the CPU and launches the game
    pub fn launch_game(&mut self) {
        
        self.p_cpu.lock().unwrap().reset();
        self.total_clock = 0;
        //self.p_cpu.lock().unwrap().pc = 0xC000; // Run nestest in automation mode (Fails at C6BD because of unofficial opcode)
        loop {
            // CPU is clocked every 3 PPU cycles
            if self.total_clock%3 == 0 {
                // If we initialized a DMA, do not clock CPU for nearly 513 cycles
                if self.p_ppu.lock().unwrap().registers.perform_dma {
                    self.perform_dma();
                }
                else {
                    self.p_cpu.lock().unwrap().clock();
                }
            }

            // Clock PPU
            self.p_ppu.lock().unwrap().clock();

            // Check if an NMI interrupt should be thrown
            if self.p_ppu.lock().unwrap().registers.emit_nmi {
                self.p_ppu.lock().unwrap().registers.emit_nmi = false;
                self.p_cpu.lock().unwrap().interrupt(Interrupt::NMI);
            }

            self.total_clock += 1;
        }
    }

    // Performs a DMA (transfer of 256 bytes of sprite data to PPU)
    pub fn perform_dma(&mut self) {
        if !self.dma_started {
            // Wait for an even cycle to start
            if self.total_clock % 2 == 0 {
                self.dma_hi_address = self.p_ppu.lock().unwrap().registers.oam_dma;
                self.dma_base_address = self.p_ppu.lock().unwrap().registers.oam_addr;
                self.dma_address = self.dma_base_address;
                self.dma_started = true;
            }
        }
        else {
            // On even cycles, read data from the bus
            if self.total_clock % 2 == 1 {
                let address: u16 = (self.dma_address as u16) + ((self.dma_hi_address as u16) << 8);
                self.dma_data = self.p_bus.lock().unwrap().read(address);
            }
            // On odd cycles, write data to the PPU OAM
            else {
                self.p_ppu.lock().unwrap().oam.write_primary(self.dma_address, self.dma_data);

                if self.dma_address < 255 {
                    self.dma_address += 1;
                }
                else {
                    self.dma_address = 0;
                }

                // End DMA
                if self.dma_address == self.dma_base_address {
                    self.dma_started = false;
                    self.p_ppu.lock().unwrap().registers.perform_dma = false;
                }
            }
        }
    }
}