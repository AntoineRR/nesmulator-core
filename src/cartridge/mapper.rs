use super::cartridge::Cartridge;

// Implements different mappers

#[derive(Debug)]
pub struct Mapper {
    pub number: u8,
    pub mirroring: bool,
    pub cartridge: Cartridge,
    pub ram: [u8;0x2000] // Additionnal RAM located on the cartridge
}

impl Mapper {
    pub fn new(cartridge: Cartridge) -> Self {
        let number: u8 = cartridge.header.control_1 >> 4 + (cartridge.header.control_2 >> 4) << 4;
        let mirroring: bool = (cartridge.header.control_1 & 0x01) == 1;
        
        Mapper {
            number,
            mirroring,
            cartridge,
            ram: [0;0x2000]
        }
    }

    // ===== READS AND WRITES FROM THE BUS =====

    pub fn bus_read(&self, address: u16) -> u8 {
        match self.number {
            0 => self.bus_read_000(address),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    pub fn bus_read_000(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => value = 0,//panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => value = self.ram[(address & 0x1FFF) as usize],
            0x8000..=0xBFFF => value = self.cartridge.prg_rom[0][(address & 0x3FFF) as usize],
            0xC000..=0xFFFF => value = self.cartridge.prg_rom[(self.cartridge.header.n_prg_rom - 1) as usize][(address & 0x3FFF) as usize]
        }
        value
    }

    pub fn bus_write(&mut self, address: u16, value: u8) {
        match self.number {
            0 => self.bus_write_000(address, value),
            _ => panic!("Invalid mapper number : {} !",self.number)
        }
    }

    pub fn bus_write_000(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => self.ram[(address & 0x1FFF) as usize] = value,
            0x8000..=0xFFFF => panic!("Tried to write to the PRG ROM : {:#X}",address)
        }
    }

    // ===== READS AND WRITES FROM THE PPU =====

    pub fn ppu_read(&self, address: u16) -> u8 {
        match self.number {
            0 => self.ppu_read_000(address),
            _ => panic!("Invalid mapper number : {}",self.number)
        }
    }

    pub fn ppu_read_000(&self, address: u16) -> u8 {
        // There is only one CHR ROM
        self.cartridge.chr_rom[0][address as usize]
    }

    pub fn ppu_write(&mut self, address: u16, value: u8) {
        match self.number {
            0 => self.ppu_write_000(address, value),
            _ => panic!("Invalid mapper number : {}",self.number)
        }
    }

    pub fn ppu_write_000(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.cartridge.chr_rom[0][address as usize] = value;
            }
            _ => panic!("Invalid address given to PPU bus : {:#X}",address)
        }
    }
}