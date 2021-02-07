// Mapper 0 : NROM

use super::mapper::{Mapper, Mirroring};

#[derive(Clone)]
pub struct Mapper0 {
    pub mirroring: Mirroring,
    pub prg_rom: Vec<[u8;16*1024]>,
    pub chr_rom: Vec<[u8;8*1024]>,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<[u8;16*1024]>, chr_rom: Vec<[u8;8*1024]>, mirroring: Mirroring) -> Self {
        Mapper0 {
            mirroring,
            prg_rom,
            chr_rom,
        }
    }
}

impl Mapper for Mapper0 {
    fn prg_rom_read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x8000..=0xBFFF => value = self.prg_rom[0][(address & 0x3FFF) as usize],
            0xC000..=0xFFFF => value = self.prg_rom[(self.prg_rom.len() - 1) as usize][(address & 0x3FFF) as usize]
        }
        value
    }

    fn prg_rom_write(&mut self, address: u16, _value: u8) {
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => panic!("Mapper 0 doesn't use this address : {:#X}",address),
            0x8000..=0xFFFF => panic!("Tried to write to the PRG ROM : {:#X}",address)
        }
    }

    fn chr_rom_read(&self, address: u16) -> u8 {
        self.chr_rom[0][address as usize]
    }

    fn chr_rom_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.chr_rom[0][address as usize] = value,
            _ => panic!("Invalid address given to PPU bus : {:#X}",address)
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn box_clone(&self) -> Box<dyn Mapper> {
        Box::new((*self).clone())
    }
}