// Reprensents the PPU data bus

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};

use crate::cartridge::mapper::Mapper;

use super::enums::VRAMAddressMask;

// ===== STRUCT =====

#[derive(Debug, Clone, Copy)]
pub struct VRAMAddress {
    pub address: u16
}

impl VRAMAddress {
    pub fn new() -> Self {
        VRAMAddress {
            address: 0
        }
    }

    pub fn get_address_part(&self, mask: VRAMAddressMask) -> u16 {
        match mask {
            VRAMAddressMask::CoarseXScroll => self.address & mask as u16,
            VRAMAddressMask::CoarseYScroll => (self.address & mask as u16) >> 5,
            VRAMAddressMask::NametableSelect => (self.address & mask as u16) >> 10,
            VRAMAddressMask::NametableX => (self.address & mask as u16) >> 10,
            VRAMAddressMask::NametableY => (self.address & mask as u16) >> 11,
            VRAMAddressMask::FineY => (self.address & mask as u16) >> 12,
            _ => 0 // We won't need to get the other VRAMAddressMask
        }
    }

    pub fn set_address_part(&mut self, mask: VRAMAddressMask, value: u16) {
        match mask {
            VRAMAddressMask::CoarseXScroll => self.address = (self.address & !(mask as u16)) | value,
            VRAMAddressMask::CoarseYScroll => self.address = (self.address & !(mask as u16)) | (value << 5),
            VRAMAddressMask::NametableSelect => self.address = (self.address & !(mask as u16)) | (value << 10),
            VRAMAddressMask::NametableX => self.address = (self.address & !(mask as u16)) | (value << 10),
            VRAMAddressMask::NametableY => self.address = (self.address & !(mask as u16)) | (value << 11),
            VRAMAddressMask::FineY => self.address = (self.address & !(mask as u16)) | (value << 12),
            VRAMAddressMask::FW2006 => self.address = (self.address & !(mask as u16)) | (value << 8),
            VRAMAddressMask::SW2006 => self.address = (self.address & !(mask as u16)) | value
        }
    }
}

#[derive(Debug)]
pub struct PPUBus {
    // Pattern tables
    pub pattern_tables: [[u8;0x1000];2],

    // Name tables loaded in VRAM
    pub name_tables: [[u8;0x0400];2],

    // Palette table
    pub palette_table: [u8;0x20],

    // Registers
    pub vram_address: VRAMAddress,
    pub tmp_vram_address: VRAMAddress,

    // Mapper
    pub o_p_mapper: Option<Arc<Mutex<Mapper>>>
}

impl PPUBus {
    pub fn new() -> Self {
        PPUBus {
            pattern_tables: [[0;0x1000];2],

            name_tables: [[0;0x0400];2],

            palette_table: [0;0x20],

            vram_address: VRAMAddress::new(),
            tmp_vram_address: VRAMAddress::new(),

            o_p_mapper: None
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x1FFF => value = self.o_p_mapper.as_ref().unwrap().lock().unwrap().ppu_read(address),
            0x2000..=0x2FFF => value = self.read_name_tables(address),
            0x3000..=0x3EFF => value = self.read_name_tables(address & 0x2FFF),
            0x3F00..=0x3FFF => value = self.read_palette_table(address & 0x001F),
            _ => panic!("Invalid address given to PPU : {:#X}",address)
        }
        value
    }

    pub fn read_name_tables(&self, address: u16) -> u8 {
        let value: u8;
        // Vertical mirroring
        if self.o_p_mapper.as_ref().unwrap().lock().unwrap().mirroring {
            match address {
                0x2000..=0x23FF => value = self.name_tables[0][(address & 0x03FF) as usize],
                0x2400..=0x27FF => value = self.name_tables[1][(address & 0x03FF) as usize],
                0x2800..=0x2BFF => value = self.name_tables[0][(address & 0x03FF) as usize],
                0x2C00..=0x2FFF => value = self.name_tables[1][(address & 0x03FF) as usize],
                _ => panic!("Invalid nametable address : {:#X}",address)
            }
        }
        // Horizontal mirroring or controlled by the mapper
        else {
            match address {
                0x2000..=0x23FF => value = self.name_tables[0][(address & 0x03FF) as usize],
                0x2400..=0x27FF => value = self.name_tables[0][(address & 0x03FF) as usize],
                0x2800..=0x2BFF => value = self.name_tables[1][(address & 0x03FF) as usize],
                0x2C00..=0x2FFF => value = self.name_tables[1][(address & 0x03FF) as usize],
                _ => panic!("Invalid nametable address : {:#X}",address)
            }
        }
        value
    }

    pub fn read_palette_table(&self, address: u16) -> u8 {
        let mut index: u16 = address;
        match index {
            0x0010 => index = 0x0000,
            0x0014 => index = 0x0004,
            0x0018 => index = 0x0008,
            0x001C => index = 0x000C,
            _ => ()
        }
        self.palette_table[index as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.o_p_mapper.as_mut().unwrap().lock().unwrap().ppu_write(address, value),
            0x2000..=0x2FFF => self.write_name_tables(address, value),
            0x3000..=0x3EFF => self.write_name_tables(address & 0x2FFF, value),
            0x3F00..=0x3FFF => self.write_palette_table(address & 0x001F, value),
            _ => panic!("Invalid address given to PPU : {:#X}",address)
        }
    }

    pub fn write_name_tables(&mut self, address: u16, value: u8) {
        // Vertical mirroring
        if self.o_p_mapper.as_ref().unwrap().lock().unwrap().mirroring {
            match address {
                0x2000..=0x23FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2400..=0x27FF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                0x2800..=0x2BFF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2C00..=0x2FFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                _ => panic!("Invalid nametable address : {:#X}",address)
            }
        }
        // Horizontal mirroring or controlled by the mapper
        else {
            match address {
                0x2000..=0x23FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2400..=0x27FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2800..=0x2BFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                0x2C00..=0x2FFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                _ => panic!("Invalid nametable address : {:#X}",address)
            }
        }
    }

    pub fn write_palette_table(&mut self, address: u16, value: u8) {
        let mut index: u16 = address;
        match index {
            0x0010 => index = 0x0000,
            0x0014 => index = 0x0004,
            0x0018 => index = 0x0008,
            0x001C => index = 0x000C,
            _ => ()
        }
        self.palette_table[index as usize] = value;
    }
}