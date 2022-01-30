// Represents a cartridge, loaded from a file
// The only supported file format is iNES (.nes)

// ===== IMPORTS =====

use std::{error::Error, fs::File, io::Read, path::Path, vec};

use log::{debug, info};

use super::{
    mapper::{Mapper, Mirroring},
    mapper_000::Mapper0,
    mapper_001::Mapper1,
    mapper_002::Mapper2,
    mapper_003::Mapper3,
};

// Header of the iNES format
#[derive(Debug)]
pub struct INesHeader {
    n_prg_rom: u8,
    n_chr_rom: u8,
    mapper_number: u8,
    mirroring: Mirroring,
}

impl INesHeader {
    pub fn new(buffer: [u8; 16]) -> Result<Self, Box<dyn Error>> {
        if buffer[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
            Err("Invalid iNES format")?;
        }

        let n_prg_rom = buffer[4];
        let n_chr_rom = buffer[5];

        let mapper_number: u8 = (buffer[6] >> 4) + ((buffer[7] >> 4) << 4);

        let mirroring = match (buffer[6] & 0x01 > 0, buffer[6] & 0x08 > 0) {
            (false, false) => Mirroring::Horizontal,
            (true, false) => Mirroring::Vertical,
            (_, true) => Mirroring::FourScreens,
        };

        Ok(INesHeader {
            n_prg_rom,
            n_chr_rom,
            mapper_number,
            mirroring,
        })
    }
}

pub struct Cartridge {
    pub mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        info!("Loading {}", path.display());

        // Opens file in read only mode
        let mut file = File::open(path)?;

        // Reads the first 16 bytes of the file
        // This is the header of the file
        let mut buffer: [u8; 16] = [0; 16];
        file.read(&mut buffer)?;
        let header = INesHeader::new(buffer)?;

        debug!(
            "{} 16KB PRG ROM units | {} 8KB CHR ROM units",
            header.n_prg_rom, header.n_chr_rom
        );

        // Stores the prg_rom
        let mut prg_rom = vec![];
        let mut buffer = [0; 16 * 1024];
        for _i in 0..header.n_prg_rom {
            file.read(&mut buffer)?;
            prg_rom.push(buffer);
        }

        // Stores the chr_rom
        let mut chr_rom = vec![];
        let mut buffer = [0; 8 * 1024];
        for _i in 0..header.n_chr_rom {
            file.read(&mut buffer)?;
            chr_rom.push(buffer);
        }
        if chr_rom.len() == 0 {
            chr_rom.push(buffer);
        }

        let mapper: Box<dyn Mapper> = match header.mapper_number {
            0 => Box::new(Mapper0::new(prg_rom, chr_rom, header.mirroring)),
            1 => Box::new(Mapper1::new(prg_rom, chr_rom, header.mirroring)),
            2 => Box::new(Mapper2::new(prg_rom, chr_rom, header.mirroring)),
            3 => Box::new(Mapper3::new(prg_rom, chr_rom, header.mirroring)),
            x => panic!("Mapper {} is not implemented", x),
        };

        info!("Using mapper {}", header.mapper_number);

        Ok(Cartridge { mapper })
    }
}
