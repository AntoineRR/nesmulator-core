use std::any::Any;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::cartridge::mapper_000::Mapper0;
use crate::cartridge::mapper_001::Mapper1;
use crate::cartridge::mapper_002::Mapper2;
use crate::cartridge::mapper_003::Mapper3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLower,
    OneScreenUpper,
    FourScreens,
}

#[typetag::serde(tag = "type")]
pub trait MapperState {
    fn as_any(&self) -> &dyn Any;
}

pub trait Mapper {
    fn prg_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>>;
    fn prg_rom_write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>>;
    fn chr_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>>;
    fn chr_rom_write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>>;
    fn get_mirroring(&self) -> Mirroring;
    fn load_persistent_memory(&mut self, _save_path: &str) -> Result<(), Box<dyn Error>> {
        Err("ROM has no persistent memory".into())
    }
    fn save_persistent_memory(&self, _save_path: &str) -> Result<(), Box<dyn Error>> {
        Err("ROM has no persistent memory".into())
    }
    fn get_mapper_state(&self) -> Box<dyn MapperState>;
    fn set_mapper_state(&mut self, state: &dyn MapperState);
}

// Header of the iNES format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct INesHeader {
    pub path_to_rom: String,

    pub n_prg_rom: u8,
    pub n_chr_rom: u8,
    pub mapper_number: u8,
    pub mirroring: Mirroring,
    pub has_persistent_memory: bool,
}

impl INesHeader {
    pub fn new(buffer: [u8; 16], path_to_rom: &str) -> Result<Self, Box<dyn Error>> {
        if buffer[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
            return Err("Invalid iNES format".into());
        }

        let n_prg_rom = buffer[4];
        let n_chr_rom = buffer[5];

        let mapper_number: u8 = (buffer[6] >> 4) + ((buffer[7] >> 4) << 4);

        let mirroring = match (buffer[6] & 0x01 > 0, buffer[6] & 0x08 > 0) {
            (false, false) => Mirroring::Horizontal,
            (true, false) => Mirroring::Vertical,
            (_, true) => Mirroring::FourScreens,
        };

        let has_persistent_memory = buffer[6] & 0x02 > 0;

        Ok(INesHeader {
            path_to_rom: String::from(path_to_rom),
            n_prg_rom,
            n_chr_rom,
            mapper_number,
            mirroring,
            has_persistent_memory,
        })
    }
}

pub fn get_mapper(path: &str) -> Result<Box<dyn Mapper>, Box<dyn Error>> {
    // Opens file in read only mode
    let mut file = File::open(path)?;

    // Reads the first 16 bytes of the file
    // This is the header of the file
    let mut buffer: [u8; 16] = [0; 16];
    file.read_exact(&mut buffer)?;
    let header = INesHeader::new(buffer, path)?;

    debug!(
        "{} 16KB PRG ROM units | {} 8KB CHR ROM units",
        header.n_prg_rom, header.n_chr_rom
    );

    // Stores the prg_rom
    let mut prg_rom = vec![];
    let mut buffer = [0; 16 * 1024];
    for _i in 0..header.n_prg_rom {
        file.read_exact(&mut buffer)?;
        prg_rom.push(buffer);
    }

    // Stores the chr_rom
    let mut chr_rom = vec![];
    let mut buffer = [0; 8 * 1024];
    for _i in 0..header.n_chr_rom {
        file.read_exact(&mut buffer)?;
        chr_rom.push(buffer);
    }
    if chr_rom.is_empty() {
        chr_rom.push(buffer);
    }

    // Create Mapper
    let mapper_number = header.mapper_number;
    let mapper: Box<dyn Mapper> = match mapper_number {
        0 => Box::new(Mapper0::new(prg_rom, chr_rom, header)),
        1 => Box::new(Mapper1::new(prg_rom, chr_rom, header)),
        2 => Box::new(Mapper2::new(prg_rom, chr_rom, header)),
        3 => Box::new(Mapper3::new(prg_rom, chr_rom, header)),
        x => panic!("Mapper {} is not implemented", x),
    };

    debug!("Using mapper {}", mapper_number);

    Ok(mapper)
}
