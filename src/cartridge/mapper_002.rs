// Mapper 2 : UNROM

use super::mapper::{Mapper, Mirroring};

#[derive(Clone)]
pub struct Mapper2 {
    pub mirroring: Mirroring,
    pub lo_prg_rom: usize,
    pub prg_rom: Vec<[u8;16*1024]>,
    pub chr_rom: Vec<[u8;8*1024]>
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<[u8;16*1024]>, chr_rom: Vec<[u8;8*1024]>, mirroring: Mirroring) -> Self {
        Mapper2 {
            mirroring,
            lo_prg_rom: 0,
            prg_rom,
            chr_rom
        }
    }
}

impl Mapper for Mapper2 {
    fn prg_rom_read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 2 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => value = 0,//panic!("Mapper 2 doesn't use this address : {:#X}",address),
            0x8000..=0xBFFF => value = self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize],
            0xC000..=0xFFFF => value = self.prg_rom[(self.prg_rom.len() - 1) as usize][(address & 0x3FFF) as usize]
        }
        value
    }

    fn prg_rom_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 2 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => panic!("Mapper 2 doesn't use this address : {:#X}",address),
            0x8000..=0xFFFF => self.lo_prg_rom = (value & 0x0F) as usize
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