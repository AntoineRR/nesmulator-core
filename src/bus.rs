// Represents the CPU bus of the NES

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use cartridge::mapper::Mapper;

use crate::{cartridge, ppu::ppu::PPU};

// ===== CONSTANTS =====

pub const STACK_OFFSET: u16 = 0x100;

// ===== BUS STRUCT =====

#[derive(Debug)]
pub struct Bus {
    pub data: [u8;0x10000],
    pub mapper: Arc<Mutex<Option<Mapper>>>,
    pub p_ppu: Arc<Mutex<PPU>>
}

impl Bus {
    pub fn new(p_ppu: Arc<Mutex<PPU>>) -> Self {
        Bus {
            data: [0;0x10000], // 64KB of ram
            mapper: Arc::new(Mutex::new(None)),
            p_ppu
        }
    }

    // Reads data from the bus at the specified address
    pub fn read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => value = self.data[address as usize],
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => value = self.data[(address & 0x07FF) as usize],
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => value = self.p_ppu.lock().unwrap().read(address),
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => value = self.p_ppu.lock().unwrap().read(address & 0x2007),
            // 0x4000 - 0x4020 / NES APU I/O Registers
            0x4000..=0x4020 => value = self.data[address as usize],
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => value = self.mapper.lock().unwrap().as_ref().unwrap().read_data(address)
        }
        value
    }

    // Writes data to the bus at the specified address
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => self.data[address as usize] = value,
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => self.data[(address & 0x07FF) as usize] = value,
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => self.p_ppu.lock().unwrap().write(address, value),
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => self.p_ppu.lock().unwrap().write(address & 0x2007, value),
            // 0x4000 - 0x4020 / NES APU I/O Registers
            0x4000..=0x4020 => self.data[address as usize] = value,
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => self.mapper.lock().unwrap().as_mut().unwrap().write_data(address, value)
        }
    }
}