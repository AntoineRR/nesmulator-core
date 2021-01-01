// Represents the CPU bus of the NES

// ===== IMPORTS =====

use cartridge::mapper::Mapper;

use crate::cartridge;

// ===== CONSTANTS =====

pub const STACK_OFFSET: u16 = 0x100;

// ===== BUS STRUCT =====

#[derive(Debug, Clone, Copy)]
pub struct Bus {
    pub data: [u8;0x10000],
    pub mapper: Mapper
}

impl Bus {
    pub const fn new() -> Self {
        Bus {
            data: [0;0x10000], // 64KB of ram
            mapper: Mapper::new(0)
        }
    }

    // Reads data from the bus at the specified address
    pub fn read(self, address: u16) -> u8 {
        let value: u8;
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => value = self.data[address as usize],
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => value = self.data[(address & 0x07FF) as usize],
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => value = self.data[address as usize],
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => value = self.data[(address & 0x2007) as usize],
            // 0x4000 - 0x4020 / NES APU I/O Registers
            0x4000..=0x4020 => value = self.data[address as usize],
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => value = self.mapper.read_data(address)
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
            0x2000..=0x2007 => self.data[address as usize] = value,
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => self.data[(address & 0x2007) as usize] = value,
            // 0x4000 - 0x4020 / NES APU I/O Registers
            0x4000..=0x4020 => self.data[address as usize] = value,
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => self.mapper.write_data(address, value)
        }
    }
}