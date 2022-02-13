// Mapper 1 : MMC1

use std::convert::TryInto;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use log::{debug, warn};

use super::mapper::{INesHeader, Mapper, Mirroring};

#[derive(Debug)]
enum PrgRomBankMode {
    Switch32,
    Switch16FirstFixed,
    Switch16LastFixed,
}

#[derive(Debug)]
enum ChrRomBankMode {
    Switch8,
    Switch4,
}

pub struct Mapper1 {
    header: INesHeader,

    lo_prg_rom: usize,
    hi_prg_rom: usize,
    lo_chr_rom: usize,
    hi_chr_rom: usize,

    prg_rom: Vec<[u8; 0x4000]>,
    chr_rom: Vec<[u8; 0x1000]>,

    ram: [u8; 0x2000],
    ram_disabled: bool,

    shift_register: u8,
    n_bit_loaded: u8,

    control_register: u8,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<[u8; 0x4000]>, chr_rom: Vec<[u8; 0x2000]>, header: INesHeader) -> Self {
        let mut converted: Vec<[u8; 0x1000]> = vec![];
        for elt in chr_rom.iter() {
            converted.push(elt[0..0x1000].try_into().expect("Failed to convert array"));
            converted.push(
                elt[0x1000..0x2000]
                    .try_into()
                    .expect("Failed to convert array"),
            );
        }
        // Add additionnal CHR ROM to get 32 banks (max for MMC1)
        while converted.len() < 0x20 {
            converted.push([0; 0x1000]);
        }

        Mapper1 {
            header,
            lo_prg_rom: 0,
            hi_prg_rom: prg_rom.len() - 1,
            lo_chr_rom: 0,
            hi_chr_rom: 1,
            prg_rom,
            chr_rom: converted,
            ram: [0; 0x2000],
            ram_disabled: false,
            shift_register: 0,
            n_bit_loaded: 0,
            control_register: 0x1C,
        }
    }

    fn get_prg_rom_bank_mode(&self) -> PrgRomBankMode {
        match (self.control_register & 0x0C) >> 2 {
            0 | 1 => PrgRomBankMode::Switch32,
            2 => PrgRomBankMode::Switch16FirstFixed,
            3 => PrgRomBankMode::Switch16LastFixed,
            _ => panic!("Unreachable pattern"),
        }
    }

    fn get_chr_rom_bank_mode(&self) -> ChrRomBankMode {
        match (self.control_register & 0x10) >> 4 {
            0 => ChrRomBankMode::Switch8,
            1 => ChrRomBankMode::Switch4,
            _ => panic!("Unreachable pattern"),
        }
    }

    fn get_path_to_save(&self) -> PathBuf {
        let path_to_rom = Path::new(&self.header.path_to_rom);
        path_to_rom
            .parent()
            .unwrap()
            .join(path_to_rom.file_stem().unwrap())
            .with_extension("sav")
    }
}

impl Mapper for Mapper1 {
    fn prg_rom_read(&self, address: u16) -> u8 {
        let value: u8;
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}", address),
            0x4020..=0x5FFF => value = 0, // panic!("Mapper 1 doesn't use this address : {:#X}", address),
            0x6000..=0x7FFF => value = self.ram[(address & 0x1FFF) as usize],
            0x8000..=0xBFFF => match self.get_prg_rom_bank_mode() {
                PrgRomBankMode::Switch32 => {
                    value = self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize]
                }
                PrgRomBankMode::Switch16FirstFixed => {
                    value = self.prg_rom[0][(address & 0x3FFF) as usize]
                }
                PrgRomBankMode::Switch16LastFixed => {
                    value = self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize]
                }
            },
            0xC000..=0xFFFF => match self.get_prg_rom_bank_mode() {
                PrgRomBankMode::Switch32 => {
                    value = self.prg_rom[self.lo_prg_rom + 1][(address & 0x3FFF) as usize]
                }
                PrgRomBankMode::Switch16FirstFixed => {
                    value = self.prg_rom[self.hi_prg_rom][(address & 0x3FFF) as usize]
                }
                PrgRomBankMode::Switch16LastFixed => {
                    value = self.prg_rom[self.prg_rom.len() - 1][(address & 0x3FFF) as usize]
                }
            },
        }
        value
    }

    fn prg_rom_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x401F => panic!("Invalid address given to mapper : {:#X}", address),
            0x4020..=0x5FFF => warn!("Mapper 1 doesn't use this address : {:#X}", address),
            0x6000..=0x7FFF => self.ram[(address & 0x1FFF) as usize] = value,
            0x8000..=0xFFFF => {
                if value & 0x80 > 0 {
                    self.shift_register = 0;
                    self.n_bit_loaded = 0;
                    self.control_register |= 0x0C;
                    debug!("CHR bank mode : {:?}", self.get_chr_rom_bank_mode());
                    debug!("PRG bank mode : {:?}", self.get_prg_rom_bank_mode());
                    debug!("Mirroring mode : {:?}", self.get_mirroring());
                } else {
                    self.shift_register >>= 1;
                    self.shift_register |= (value & 0x01) << 4;
                    self.n_bit_loaded += 1;
                    if self.n_bit_loaded == 5 {
                        let action = (address & 0x6000) >> 13;
                        match action {
                            0 => {
                                self.control_register = self.shift_register & 0x1F;
                                debug!("CHR bank mode : {:?}", self.get_chr_rom_bank_mode());
                                debug!("PRG bank mode : {:?}", self.get_prg_rom_bank_mode());
                                debug!("Mirroring mode : {:?}", self.get_mirroring());
                            }
                            1 => match self.get_chr_rom_bank_mode() {
                                ChrRomBankMode::Switch8 => {
                                    self.lo_chr_rom = (self.shift_register & 0x1E) as usize
                                }
                                ChrRomBankMode::Switch4 => {
                                    self.lo_chr_rom = (self.shift_register & 0x1F) as usize
                                }
                            },
                            2 => self.hi_chr_rom = (self.shift_register & 0x1F) as usize,
                            3 => {
                                self.ram_disabled = (self.shift_register & 0x10) > 0;
                                match self.get_prg_rom_bank_mode() {
                                    PrgRomBankMode::Switch32 => {
                                        self.lo_prg_rom = (self.shift_register & 0x0E) as usize
                                    }
                                    PrgRomBankMode::Switch16FirstFixed => {
                                        self.hi_prg_rom = (self.shift_register & 0x0F) as usize
                                    }
                                    PrgRomBankMode::Switch16LastFixed => {
                                        self.lo_prg_rom = (self.shift_register & 0x0F) as usize
                                    }
                                }
                            }
                            _ => panic!("Unreachable pattern"),
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
            ChrRomBankMode::Switch8 => match address {
                0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize],
                0x1000..=0x1FFF => self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize],
                _ => panic!("Invalid address given to PPU bus : {:#X}", address),
            },
            ChrRomBankMode::Switch4 => match address {
                0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize],
                0x1000..=0x1FFF => self.chr_rom[self.hi_chr_rom][(address & 0x0FFF) as usize],
                _ => panic!("Invalid address given to PPU bus : {:#X}", address),
            },
        }
    }

    fn chr_rom_write(&mut self, address: u16, value: u8) {
        match self.get_chr_rom_bank_mode() {
            ChrRomBankMode::Switch8 => match address {
                0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize] = value,
                0x1000..=0x1FFF => {
                    self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize] = value
                }
                _ => panic!("Invalid address given to PPU bus : {:#X}", address),
            },
            ChrRomBankMode::Switch4 => match address {
                0x0000..=0x0FFF => self.chr_rom[self.lo_chr_rom][address as usize] = value,
                0x1000..=0x1FFF => {
                    self.chr_rom[self.hi_chr_rom][(address & 0x0FFF) as usize] = value
                }
                _ => panic!("Invalid address given to PPU bus : {:#X}", address),
            },
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        match self.control_register & 0x03 {
            0 => Mirroring::OneScreenLower,
            1 => Mirroring::OneScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => panic!("Unreachable pattern"),
        }
    }

    fn load_persistent_memory(&mut self) -> Result<(), Box<dyn Error>> {
        if self.header.has_persistent_memory {
            let path_to_save = self.get_path_to_save();
            if path_to_save.exists() {
                self.ram = fs::read(path_to_save)?[..].try_into()?;
                return Ok(());
            }
            return Err(format!("Save file {} not found", path_to_save.to_str().unwrap()).into());
        }
        Err("ROM has no persistent memory".into())
    }

    fn save_persistent_memory(&self) -> Result<(), Box<dyn Error>> {
        if self.header.has_persistent_memory {
            let mut save_file = File::create(self.get_path_to_save())?;
            save_file.write_all(&self.ram)?;
            return Ok(());
        }
        Err("ROM has no persistent memory".into())
    }
}
