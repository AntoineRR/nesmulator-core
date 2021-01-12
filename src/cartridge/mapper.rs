use super::cartridge::Cartridge;

// Implements different mappers

#[derive(Debug)]
pub struct Mapper {
    pub number: u8,
    pub mirroring: bool,
    pub cartridge: Cartridge
}

impl Mapper {
    pub fn new(cartridge: Cartridge) -> Self {
        let number: u8 = cartridge.header.control_1 >> 4 + (cartridge.header.control_2 >> 4) << 4;
        let mirroring: bool = (cartridge.header.control_1 & 0x01) == 1;
        
        Mapper {
            number,
            mirroring,
            cartridge
        }
    }

    pub fn read_prg(&self, address: u16) -> u8 {
        match self.number {
            0 => self.read_prg_0(address),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    pub fn read_chr(&self, address: u16) -> u8 {
        match self.number {
            0 => self.read_chr_0(address),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    // Not sure it is useful
    // pub fn write_prg(&mut self, address: u16, data: u8) {
    //     match self.number {
    //         _ => panic!("Invalid mapper number : {} !",self.number)
    //     }
    // }

    // Mapper 0 read PRG ROM method
    pub fn read_prg_0(&self, address: u16) -> u8 {
        // If there is only 1 PRG ROM
        if self.cartridge.header.n_prg_rom == 1 {
            self.cartridge.prg_rom[0][(address & 0x3FFF) as usize]
        }
        // Else there are 2
        else {
            if address < 0xC000 {
                self.cartridge.prg_rom[0][(address & 0x3FFF) as usize]
            }
            else {
                self.cartridge.prg_rom[1][(address & 0x3FFF) as usize]
            }
        }
    }

    // Mapper 0 read CHR ROM method
    pub fn read_chr_0(&self, address: u16) -> u8 {
        // There is only one CHR ROM
        self.cartridge.chr_rom[0][address as usize]
    }
}