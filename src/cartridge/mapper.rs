use crate::CARTRIDGE;

// Implements different mappers

#[derive(Debug, Clone, Copy)]
pub struct Mapper {
    pub number: u8,
}

impl Mapper {
    pub const fn new(_number: u8) -> Self {
        Mapper {
            number: _number
        }
    }

    pub fn read_data(&self, address: u16) -> u8 {
        match self.number {
            0 => self.read_data_0(address),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    pub fn write_data(&self, address: u16, data: u8) {
        match self.number {
            0 => self.write_data_0(address, data),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    // Mapper 0 read method
    pub fn read_data_0(&self, address: u16) -> u8 {
        unsafe {
            // If there is only 1 PRG ROM
            if CARTRIDGE.as_ref().unwrap().header.n_prg_rom == 1 {
                return CARTRIDGE.as_ref().unwrap().prg_rom[0][(address & 0x3FFF) as usize];
            }
            // Else there are 2
            else {
                if address < 0xC000 {
                    return CARTRIDGE.as_ref().unwrap().prg_rom[0][(address & 0x3FFF) as usize];
                }
                else {
                    return CARTRIDGE.as_ref().unwrap().prg_rom[1][(address & 0x3FFF) as usize];
                }
            }
        }
    }

    // Mapper 0 write method
    pub fn write_data_0(&self, address: u16, data: u8) {
        unsafe {
            // If there is only 1 PRG ROM
            if CARTRIDGE.as_ref().unwrap().header.n_prg_rom == 1 {
                CARTRIDGE.as_mut().unwrap().prg_rom[0][(address & 0x3FFF) as usize] = data;
            }
            // Else there are 2
            else {
                if address < 0xC000 {
                    CARTRIDGE.as_mut().unwrap().prg_rom[0][(address & 0x3FFF) as usize] = data;
                }
                else {
                    CARTRIDGE.as_mut().unwrap().prg_rom[1][(address & 0x3FFF) as usize] = data;
                }
            }
        }
    }
}