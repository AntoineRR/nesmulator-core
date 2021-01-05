use super::cartridge::Cartridge;

// Implements different mappers

#[derive(Debug)]
pub struct Mapper {
    pub number: u8,
    pub cartridge: Cartridge
}

impl Mapper {
    pub fn new(cartridge: Cartridge) -> Self {
        let number: u8 = cartridge.header.control_1 >> 4 + (cartridge.header.control_2 >> 4) << 4;
        
        Mapper {
            number,
            cartridge
        }
    }

    pub fn read_data(&self, address: u16) -> u8 {
        match self.number {
            0 => self.read_data_0(address),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    pub fn write_data(&mut self, address: u16, data: u8) {
        match self.number {
            0 => self.write_data_0(address, data),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    // Mapper 0 read method
    pub fn read_data_0(&self, address: u16) -> u8 {
        // If there is only 1 PRG ROM
        if self.cartridge.header.n_prg_rom == 1 {
            return self.cartridge.prg_rom[0][(address & 0x3FFF) as usize];
        }
        // Else there are 2
        else {
            if address < 0xC000 {
                return self.cartridge.prg_rom[0][(address & 0x3FFF) as usize];
            }
            else {
                return self.cartridge.prg_rom[1][(address & 0x3FFF) as usize];
            }
        }
    }

    // Mapper 0 write method
    pub fn write_data_0(&mut self, address: u16, data: u8) {
        // If there is only 1 PRG ROM
        if self.cartridge.header.n_prg_rom == 1 {
            self.cartridge.prg_rom[0][(address & 0x3FFF) as usize] = data;
        }
        // Else there are 2
        else {
            if address < 0xC000 {
                self.cartridge.prg_rom[0][(address & 0x3FFF) as usize] = data;
            }
            else {
                self.cartridge.prg_rom[1][(address & 0x3FFF) as usize] = data;
            }
        }
    }
}