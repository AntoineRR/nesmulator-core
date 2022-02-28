// Mapper 1 : MMC1

use std::any::Any;
use std::convert::TryInto;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::errors::{InvalidMapperReadError, InvalidMapperWriteError};
use crate::state::Stateful;

use super::mapper::{INesHeader, Mapper, MapperState, Mirroring};

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
            _ => unreachable!(),
        }
    }

    fn get_chr_rom_bank_mode(&self) -> ChrRomBankMode {
        match (self.control_register & 0x10) >> 4 {
            0 => ChrRomBankMode::Switch8,
            1 => ChrRomBankMode::Switch4,
            _ => unreachable!(),
        }
    }
}

impl Mapper for Mapper1 {
    fn prg_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x0000..=0x401F => Err(Box::new(InvalidMapperReadError(address))),
            0x4020..=0x5FFF => Err(Box::new(InvalidMapperReadError(address))),
            0x6000..=0x7FFF => Ok(self.ram[(address & 0x1FFF) as usize]),
            0x8000..=0xBFFF => match self.get_prg_rom_bank_mode() {
                PrgRomBankMode::Switch32 => {
                    Ok(self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize])
                }
                PrgRomBankMode::Switch16FirstFixed => {
                    Ok(self.prg_rom[0][(address & 0x3FFF) as usize])
                }
                PrgRomBankMode::Switch16LastFixed => {
                    Ok(self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize])
                }
            },
            0xC000..=0xFFFF => match self.get_prg_rom_bank_mode() {
                PrgRomBankMode::Switch32 => {
                    Ok(self.prg_rom[self.lo_prg_rom + 1][(address & 0x3FFF) as usize])
                }
                PrgRomBankMode::Switch16FirstFixed => {
                    Ok(self.prg_rom[self.hi_prg_rom][(address & 0x3FFF) as usize])
                }
                PrgRomBankMode::Switch16LastFixed => {
                    Ok(self.prg_rom[self.prg_rom.len() - 1][(address & 0x3FFF) as usize])
                }
            },
        }
    }

    fn prg_rom_write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match address {
            0x0000..=0x401F => Err(Box::new(InvalidMapperWriteError(address))),
            0x4020..=0x5FFF => Err(Box::new(InvalidMapperWriteError(address))),
            0x6000..=0x7FFF => {
                self.ram[(address & 0x1FFF) as usize] = value;
                Ok(())
            }
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
                            _ => unreachable!(),
                        }
                        self.shift_register = 0;
                        self.n_bit_loaded = 0;
                    }
                }
                Ok(())
            }
        }
    }

    fn chr_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match self.get_chr_rom_bank_mode() {
            ChrRomBankMode::Switch8 => match address {
                0x0000..=0x0FFF => Ok(self.chr_rom[self.lo_chr_rom][address as usize]),
                0x1000..=0x1FFF => {
                    Ok(self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize])
                }
                _ => Err(Box::new(InvalidMapperReadError(address))),
            },
            ChrRomBankMode::Switch4 => match address {
                0x0000..=0x0FFF => Ok(self.chr_rom[self.lo_chr_rom][address as usize]),
                0x1000..=0x1FFF => Ok(self.chr_rom[self.hi_chr_rom][(address & 0x0FFF) as usize]),
                _ => Err(Box::new(InvalidMapperReadError(address))),
            },
        }
    }

    fn chr_rom_write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match self.get_chr_rom_bank_mode() {
            ChrRomBankMode::Switch8 => match address {
                0x0000..=0x0FFF => {
                    self.chr_rom[self.lo_chr_rom][address as usize] = value;
                    Ok(())
                }
                0x1000..=0x1FFF => {
                    self.chr_rom[self.lo_chr_rom + 1][(address & 0x0FFF) as usize] = value;
                    Ok(())
                }
                _ => Err(Box::new(InvalidMapperWriteError(address))),
            },
            ChrRomBankMode::Switch4 => match address {
                0x0000..=0x0FFF => {
                    self.chr_rom[self.lo_chr_rom][address as usize] = value;
                    Ok(())
                }
                0x1000..=0x1FFF => {
                    self.chr_rom[self.hi_chr_rom][(address & 0x0FFF) as usize] = value;
                    Ok(())
                }
                _ => Err(Box::new(InvalidMapperWriteError(address))),
            },
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        match self.control_register & 0x03 {
            0 => Mirroring::OneScreenLower,
            1 => Mirroring::OneScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }

    fn load_persistent_memory(&mut self, save_path: &str) -> Result<(), Box<dyn Error>> {
        if self.header.has_persistent_memory {
            let path_to_save = Path::new(save_path);
            if path_to_save.exists() {
                self.ram = fs::read(path_to_save)?[..].try_into()?;
                return Ok(());
            }
            return Err(format!("Save file {} not found", path_to_save.to_str().unwrap()).into());
        }
        Err("ROM has no persistent memory".into())
    }

    fn save_persistent_memory(&self, save_path: &str) -> Result<(), Box<dyn Error>> {
        if self.header.has_persistent_memory {
            let mut save_file = File::create(save_path)?;
            save_file.write_all(&self.ram)?;
            return Ok(());
        }
        Err("ROM has no persistent memory".into())
    }

    fn get_mapper_state(&self) -> Box<dyn MapperState> {
        Box::new(self.get_state())
    }

    fn set_mapper_state(&mut self, state: &Box<dyn MapperState>) {
        match state.as_any().downcast_ref::<Mapper1State>() {
            Some(s) => self.set_state(s),
            None => panic!("State is not a Mapper1State"),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Mapper1State {
    header: INesHeader,
    lo_prg_rom: usize,
    hi_prg_rom: usize,
    lo_chr_rom: usize,
    hi_chr_rom: usize,
    #[serde_as(as = "[_; 0x2000]")]
    ram: [u8; 0x2000],
    ram_disabled: bool,
    shift_register: u8,
    n_bit_loaded: u8,
    control_register: u8,
    #[serde_as(as = "Vec<[_; 0x1000]>")]
    chr_rom: Vec<[u8; 0x1000]>,
}

#[typetag::serde]
impl MapperState for Mapper1State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Stateful for Mapper1 {
    type State = Mapper1State;

    fn get_state(&self) -> Self::State {
        Mapper1State {
            header: self.header.clone(),
            lo_prg_rom: self.lo_prg_rom,
            hi_prg_rom: self.hi_prg_rom,
            lo_chr_rom: self.lo_chr_rom,
            hi_chr_rom: self.hi_chr_rom,
            ram: self.ram,
            ram_disabled: self.ram_disabled,
            shift_register: self.shift_register,
            n_bit_loaded: self.n_bit_loaded,
            control_register: self.control_register,
            chr_rom: self.chr_rom.clone(),
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.header = state.header.clone();
        self.lo_prg_rom = state.lo_prg_rom;
        self.hi_prg_rom = state.hi_prg_rom;
        self.lo_chr_rom = state.lo_chr_rom;
        self.hi_chr_rom = state.hi_chr_rom;
        self.ram = state.ram;
        self.ram_disabled = state.ram_disabled;
        self.shift_register = state.shift_register;
        self.n_bit_loaded = state.n_bit_loaded;
        self.control_register = state.control_register;
        self.chr_rom = state.chr_rom.clone();
    }
}
