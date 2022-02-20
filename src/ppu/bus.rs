// Reprensents the PPU data bus

// ===== IMPORTS =====

use std::{cell::RefCell, error::Error, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    cartridge::mapper::{Mapper, Mirroring},
    errors::{InvalidPPUBusReadError, InvalidPPUBusWriteError},
    state::Stateful,
};

use super::{enums::VRAMAddressMask, state::PpuBusState};

type MapperRc = Rc<RefCell<Box<dyn Mapper>>>;

// ===== STRUCT =====

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VRAMAddress {
    pub address: u16,
}

impl VRAMAddress {
    pub fn new() -> Self {
        VRAMAddress { address: 0 }
    }

    pub fn get_address_part(&self, mask: VRAMAddressMask) -> u16 {
        match mask {
            VRAMAddressMask::CoarseXScroll => self.address & mask as u16,
            VRAMAddressMask::CoarseYScroll => (self.address & mask as u16) >> 5,
            VRAMAddressMask::NametableSelect => (self.address & mask as u16) >> 10,
            VRAMAddressMask::NametableX => (self.address & mask as u16) >> 10,
            VRAMAddressMask::NametableY => (self.address & mask as u16) >> 11,
            VRAMAddressMask::FineY => (self.address & mask as u16) >> 12,
            _ => 0, // We won't need to get the other VRAMAddressMask
        }
    }

    pub fn set_address_part(&mut self, mask: VRAMAddressMask, value: u16) {
        match mask {
            VRAMAddressMask::CoarseXScroll => {
                self.address = (self.address & !(mask as u16)) | value
            }
            VRAMAddressMask::CoarseYScroll => {
                self.address = (self.address & !(mask as u16)) | (value << 5)
            }
            VRAMAddressMask::NametableSelect => {
                self.address = (self.address & !(mask as u16)) | (value << 10)
            }
            VRAMAddressMask::NametableX => {
                self.address = (self.address & !(mask as u16)) | (value << 10)
            }
            VRAMAddressMask::NametableY => {
                self.address = (self.address & !(mask as u16)) | (value << 11)
            }
            VRAMAddressMask::FineY => {
                self.address = (self.address & !(mask as u16)) | (value << 12)
            }
            VRAMAddressMask::FW2006 => {
                self.address = (self.address & !(mask as u16)) | (value << 8)
            }
            VRAMAddressMask::SW2006 => self.address = (self.address & !(mask as u16)) | value,
        }
    }
}

pub struct PPUBus {
    // Name tables loaded in VRAM
    name_tables: [[u8; 0x0400]; 2],

    // Palette table
    palette_table: [u8; 0x20],

    // Registers
    pub vram_address: VRAMAddress,
    pub tmp_vram_address: VRAMAddress,

    // Mapper
    pub o_p_mapper: Option<MapperRc>,
}

impl PPUBus {
    pub fn new() -> Self {
        PPUBus {
            name_tables: [[0; 0x0400]; 2],

            palette_table: [0; 0x20],

            vram_address: VRAMAddress::new(),
            tmp_vram_address: VRAMAddress::new(),

            o_p_mapper: None,
        }
    }

    pub fn from_state(state: &PpuBusState) -> Self {
        let mut bus = PPUBus::new();
        bus.set_state(state);
        bus
    }

    pub fn set_mapper(&mut self, p_mapper: MapperRc) {
        self.o_p_mapper = Some(p_mapper);
    }

    pub fn read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x0000..=0x1FFF => self
                .o_p_mapper
                .as_ref()
                .unwrap()
                .borrow()
                .chr_rom_read(address),
            0x2000..=0x2FFF => self.read_name_tables(address),
            0x3000..=0x3EFF => self.read_name_tables(address & 0x2FFF),
            0x3F00..=0x3FFF => self.read_palette_table(address & 0x001F),
            _ => Err(Box::new(InvalidPPUBusReadError(address))),
        }
    }

    fn read_name_tables(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match self.o_p_mapper.as_ref().unwrap().borrow().get_mirroring() {
            Mirroring::Horizontal => match address {
                0x2000..=0x23FF => Ok(self.name_tables[0][(address & 0x03FF) as usize]),
                0x2400..=0x27FF => Ok(self.name_tables[0][(address & 0x03FF) as usize]),
                0x2800..=0x2BFF => Ok(self.name_tables[1][(address & 0x03FF) as usize]),
                0x2C00..=0x2FFF => Ok(self.name_tables[1][(address & 0x03FF) as usize]),
                _ => Err(Box::new(InvalidPPUBusReadError(address))),
            },
            Mirroring::Vertical => match address {
                0x2000..=0x23FF => Ok(self.name_tables[0][(address & 0x03FF) as usize]),
                0x2400..=0x27FF => Ok(self.name_tables[1][(address & 0x03FF) as usize]),
                0x2800..=0x2BFF => Ok(self.name_tables[0][(address & 0x03FF) as usize]),
                0x2C00..=0x2FFF => Ok(self.name_tables[1][(address & 0x03FF) as usize]),
                _ => Err(Box::new(InvalidPPUBusReadError(address))),
            },
            Mirroring::OneScreenLower => Ok(self.name_tables[0][(address & 0x03FF) as usize]),
            Mirroring::OneScreenUpper => Ok(self.name_tables[1][(address & 0x03FF) as usize]),
            Mirroring::FourScreens => panic!("Four screen mirroring is not handled for now"),
        }
    }

    fn read_palette_table(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        let index = match address {
            0x0010 => 0x0000,
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001C => 0x000C,
            a if a <= 0x001F => a,
            _ => return Err(Box::new(InvalidPPUBusReadError(address))),
        };
        Ok(self.palette_table[index as usize])
    }

    pub fn write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match address {
            0x0000..=0x1FFF => self
                .o_p_mapper
                .as_mut()
                .unwrap()
                .borrow_mut()
                .chr_rom_write(address, value),
            0x2000..=0x2FFF => self.write_name_tables(address, value),
            0x3000..=0x3EFF => self.write_name_tables(address & 0x2FFF, value),
            0x3F00..=0x3FFF => self.write_palette_table(address & 0x001F, value),
            _ => Err(Box::new(InvalidPPUBusWriteError(address))),
        }
    }

    fn write_name_tables(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match self.o_p_mapper.as_ref().unwrap().borrow().get_mirroring() {
            Mirroring::Horizontal => match address {
                0x2000..=0x23FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2400..=0x27FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2800..=0x2BFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                0x2C00..=0x2FFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                _ => return Err(Box::new(InvalidPPUBusWriteError(address))),
            },
            Mirroring::Vertical => match address {
                0x2000..=0x23FF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2400..=0x27FF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                0x2800..=0x2BFF => self.name_tables[0][(address & 0x03FF) as usize] = value,
                0x2C00..=0x2FFF => self.name_tables[1][(address & 0x03FF) as usize] = value,
                _ => return Err(Box::new(InvalidPPUBusWriteError(address))),
            },
            Mirroring::OneScreenLower => {
                self.name_tables[0][(address & 0x03FF) as usize] = value;
                self.name_tables[1][(address & 0x03FF) as usize] = value;
            }
            Mirroring::OneScreenUpper => {
                self.name_tables[0][(address & 0x03FF) as usize] = value;
                self.name_tables[1][(address & 0x03FF) as usize] = value;
            }
            Mirroring::FourScreens => panic!("Four screen mirroring is not handled for now"),
        }
        Ok(())
    }

    fn write_palette_table(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        let index = match address {
            0x0010 => 0x0000,
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001C => 0x000C,
            a if a <= 0x001F => a,
            _ => return Err(Box::new(InvalidPPUBusWriteError(address))),
        };
        self.palette_table[index as usize] = value;
        Ok(())
    }
}

impl Stateful for PPUBus {
    type State = PpuBusState;

    fn get_state(&self) -> Self::State {
        PpuBusState {
            name_tables: self.name_tables,
            palette_table: self.palette_table,
            vram_address: self.vram_address,
            tmp_vram_address: self.tmp_vram_address,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.name_tables = state.name_tables;
        self.palette_table = state.palette_table;
        self.vram_address = state.vram_address;
        self.tmp_vram_address = state.tmp_vram_address;
    }
}
