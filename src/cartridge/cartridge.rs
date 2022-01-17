// Represents a cartridge, loaded from a file
// The only supported file format is iNES (.nes)

// ===== IMPORTS =====

use std::{fs::File, io::Read, path::Path, vec};

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
    name: [u8; 3],
    identifier: u8,
    n_prg_rom: u8,
    n_chr_rom: u8,
    control_1: u8,
    control_2: u8,
    n_ram_banks: u8,
    unused: [u8; 7],
}

impl INesHeader {
    pub fn new() -> Self {
        INesHeader {
            name: [0; 3],
            identifier: 0,
            n_prg_rom: 0,
            n_chr_rom: 0,
            control_1: 0,
            control_2: 0,
            n_ram_banks: 0,
            unused: [0; 7],
        }
    }
}

pub struct Cartridge {
    pub mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(path: &Path) -> Self {
        info!("Loading {}", path.display());

        // Opens file in read only mode
        let mut file = match File::open(path) {
            Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
            Ok(file) => file,
        };

        // This will contain the header of the cartridge
        let mut buffer: [u8; 16] = [0; 16];

        // Reads the first 16 bytes of the file
        // This is the header of the file
        match file.read(&mut buffer) {
            Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
            Ok(_) => (),
        }

        let mut header: INesHeader = INesHeader::new();
        header.name = [buffer[0], buffer[1], buffer[2]];
        header.identifier = buffer[3];
        header.n_prg_rom = buffer[4];
        header.n_chr_rom = buffer[5];
        header.control_1 = buffer[6];
        header.control_2 = buffer[7];
        header.n_ram_banks = buffer[8];

        debug!(
            "{} PRG ROM | {} CHR ROM",
            header.n_prg_rom, header.n_chr_rom
        );

        // Stores the prg_rom
        let mut prg_rom = vec![];
        let mut buffer: [u8; 16 * 1024] = [0; 16 * 1024];
        for _i in 0..header.n_prg_rom {
            match file.read(&mut buffer) {
                Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
                Ok(_) => (),
            }
            prg_rom.push(buffer);
        }

        // Stores the chr_rom
        let mut chr_rom = vec![];
        let mut buffer: [u8; 8 * 1024] = [0; 8 * 1024];
        for _i in 0..header.n_chr_rom {
            match file.read(&mut buffer) {
                Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
                Ok(_) => (),
            }
            chr_rom.push(buffer);
        }
        if chr_rom.len() == 0 {
            chr_rom.push(buffer);
        }

        let mirroring: Mirroring = match (header.control_1 & 0x01) == 1 {
            false => Mirroring::Horizontal,
            true => Mirroring::Vertical,
        };

        let number: u8 = (header.control_1 >> 4) + ((header.control_2 >> 4) << 4);

        debug!("Using mapper {}", number);

        let mapper: Box<dyn Mapper> = match number {
            0 => Box::new(Mapper0::new(prg_rom, chr_rom, mirroring)),
            1 => Box::new(Mapper1::new(prg_rom, chr_rom, mirroring)),
            2 => Box::new(Mapper2::new(prg_rom, chr_rom, mirroring)),
            3 => Box::new(Mapper3::new(prg_rom, chr_rom, mirroring)),
            _ => panic!("Mapper {} is not implemented", number),
        };

        Cartridge { mapper }
    }
}
