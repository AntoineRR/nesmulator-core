// Mapper 1 : MMC1

use std::convert::TryInto;

use super::mapper::{Mapper, Mirroring};

enum PrgRomBankMode {
    Switch32,
    Switch16FirstFixed,
    Sxitch16LastFixed
}

enum ChrRomBankMode {
    Switch8,
    Switch4
}

#[derive(Clone)]
pub struct Mapper1 {
    pub mirroring: Mirroring,
    pub lo_prg_rom: usize,
    pub hi_prg_rom: usize,
    pub lo_chr_rom: usize,
    pub hi_chr_rom: usize,
    pub prg_rom: Vec<[u8;16*1024]>,
    pub chr_rom: Vec<[u8;4*1024]>,
    pub ram: [u8;0x2000],
    pub ram_disabled: bool,
    pub shift_register: u8,
    pub n_bit_loaded: u8,
    pub control_register: u8,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<[u8;16*1024]>, chr_rom: Vec<[u8;8*1024]>, mirroring: Mirroring) -> Self {

        let mut converted: Vec<[u8;4*1024]> = vec![];
        for elt in chr_rom.iter() {
            converted.push(elt[0..4*1024].try_into().expect("Failed to convert array"));
            converted.push(elt[4*1024..8*1024].try_into().expect("Failed to convert array"));
        }

        Mapper1 {
            mirroring,
            lo_prg_rom: 0,
            hi_prg_rom: 0,
            lo_chr_rom: 0,
            hi_chr_rom: 0,
            prg_rom,
            chr_rom: converted,
            ram: [0;0x2000],
            ram_disabled: false,
            shift_register: 0,
            n_bit_loaded: 0,
            control_register: 0
        }
    }

    fn get_prg_rom_bank_mode(&self) -> PrgRomBankMode {
        match (self.control_register & 0x0C) >> 2 {
            0 | 1 => PrgRomBankMode::Switch32,
            2 => PrgRomBankMode::Switch16FirstFixed,
            3 => PrgRomBankMode::Sxitch16LastFixed,
            _ => panic!("Unreachable pattern")
        }
    }

    fn get_chr_rom_bank_mode(&self) -> ChrRomBankMode {
        match (self.control_register & 0x10) >> 4 {
            0 => ChrRomBankMode::Switch8,
            1 => ChrRomBankMode::Switch4,
            _ => panic!("Unreachable pattern")
        }
    }
}

impl Mapper for Mapper1 {
    fn prg_rom_read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 1 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => value = self.ram[(address & 0x1FFF) as usize],
            0x8000..=0xBFFF => {
                match self.get_prg_rom_bank_mode() {
                    PrgRomBankMode::Switch32 => value = self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize],
                    PrgRomBankMode::Switch16FirstFixed => value = self.prg_rom[0][(address & 0x3FFF) as usize],
                    PrgRomBankMode::Sxitch16LastFixed => value = self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize]
                }
            }
            0xC000..=0xFFFF => {
                match self.get_prg_rom_bank_mode() {
                    PrgRomBankMode::Switch32 => value = self.prg_rom[self.lo_prg_rom + 1][(address & 0x3FFF) as usize],
                    PrgRomBankMode::Switch16FirstFixed => value = self.prg_rom[self.hi_prg_rom][(address & 0x3FFF) as usize],
                    PrgRomBankMode::Sxitch16LastFixed => value = self.prg_rom[self.prg_rom.len() - 1][(address & 0x3FFF) as usize]
                }
            }
        }
        value
    }

    fn prg_rom_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}",address),
            0x4020..=0x5FFF => panic!("Mapper 1 doesn't use this address : {:#X}",address),
            0x6000..=0x7FFF => self.ram[(address & 0x1FFF) as usize] = value,
            0x8000..=0xFFFF =>  {
                if value & 0x80 > 0 {
                    self.shift_register = 0;
                    self.n_bit_loaded = 0;
                    self.control_register |= 0x0C;
                }
                else {
                    self.shift_register >>= 1;
                    self.shift_register |= (value & 0x01) << 4;
                    self.n_bit_loaded += 1;
                    if self.n_bit_loaded == 5 {
                        match address {
                            0x8000..=0x9FFF => self.control_register = self.shift_register & 0x1F,
                            0xA000..=0xBFFF => {
                                match self.get_chr_rom_bank_mode() {
                                    ChrRomBankMode::Switch8 => self.lo_chr_rom = (self.shift_register & 0x1E) as usize,
                                    ChrRomBankMode::Switch4 => self.lo_chr_rom = (self.shift_register & 0x1F) as usize
                                }
                            },
                            0xC000..=0xDFFF => self.hi_chr_rom = (self.shift_register & 0x1F) as usize,
                            0xE000..=0xFFFF => {
                                self.ram_disabled = (self.shift_register & 0x10) > 0;
                                match self.get_prg_rom_bank_mode() {
                                    PrgRomBankMode::Switch32 => self.lo_prg_rom = (self.shift_register & 0x0E) as usize,
                                    PrgRomBankMode::Switch16FirstFixed => self.hi_prg_rom = (self.shift_register & 0x0F) as usize,
                                    PrgRomBankMode::Sxitch16LastFixed => self.lo_prg_rom = (self.shift_register & 0x0F) as usize,
                                }
                            }
                            _ => panic!("Unreachable pattern")
                        }
                        self.shift_register = 0;
                        self.n_bit_loaded = 0;
                    }
                }
            }
        }
    }

    fn chr_rom_read(&self, address: u16) -> u8 {
        match self.get_chr_rom_bank_mode() {
            ChrRomBankMode::Switch8 => {
                match address {
                    0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize],
                    0x1000..=0x1FFF => self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize],
                    _ => panic!("Invalid address given to PPU bus : {:#X}",address)
                }
            },
            ChrRomBankMode::Switch4 => {
                match address {
                    0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize],
                    0x1000..=0x1FFF => self.chr_rom[self.hi_chr_rom][(address & 0x0FFF) as usize],
                    _ => panic!("Invalid address given to PPU bus : {:#X}",address)
                }
            }
        }
    }

    fn chr_rom_write(&mut self, address: u16, value: u8) {
        match self.get_chr_rom_bank_mode() {
            ChrRomBankMode::Switch8 => {
                match address {
                    0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize] = value,
                    0x1000..=0x1FFF => self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize] = value,
                    _ => panic!("Invalid address given to PPU bus : {:#X}",address)
                }
            },
            ChrRomBankMode::Switch4 => {
                match address {
                    0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize] = value,
                    0x1000..=0x1FFF => self.chr_rom[self.lo_chr_rom][(address & 0x0FFF) as usize] = value,
                    _ => panic!("Invalid address given to PPU bus : {:#X}",address)
                }
            }
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        match self.control_register & 0x03 {
            0 => Mirroring::OneScreenLower,
            1 => Mirroring::OneScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => panic!("Unreachable pattern")
        }
    }

    fn box_clone(&self) -> Box<dyn Mapper> {
        Box::new((*self).clone())
    }
}