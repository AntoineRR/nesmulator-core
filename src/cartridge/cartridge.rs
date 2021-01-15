// Represents a cartridge, loaded from a file
// The only supported file format is iNES (.nes)

// ===== IMPORTS =====

use std::{fs::File, io::Read, path::Path, vec};

// Header of the iNES format
#[derive(Debug)]
pub struct INesHeader {
    pub name: [u8;3],
    pub identifier: u8,
    pub n_prg_rom: u8,
    pub n_chr_rom: u8,
    pub control_1: u8,
    pub control_2: u8,
    pub n_ram_banks: u8,
    pub unused: [u8;7]
}

impl INesHeader {
    pub fn new() -> Self {
        INesHeader {
            name: [0;3],
            identifier: 0,
            n_prg_rom: 0,
            n_chr_rom: 0,
            control_1: 0,
            control_2: 0,
            n_ram_banks: 0,
            unused: [0;7]
        }
    }
}

#[derive(Debug)]
pub struct Cartridge {
    pub header: INesHeader,
    pub prg_rom: Vec<[u8;16*1024]>,
    pub chr_rom: Vec<[u8;8*1024]>
}

impl Cartridge {
    pub fn new(path: &Path) -> Self {
        // Opens file in read only mode
        let mut file = match File::open(path) {
            Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
            Ok(file) => file
        };

        // This will contain the header of the cartridge
        let mut buffer: [u8;16] = [0;16];

        // Reads the first 16 bytes of the file
        // This is the header of the file
        match file.read(&mut buffer) {
            Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
            Ok(_) => ()
        }

        let mut _header: INesHeader = INesHeader::new();
        _header.name = [buffer[0],buffer[1],buffer[2]];
        _header.identifier = buffer[3];
        _header.n_prg_rom = buffer[4];
        _header.n_chr_rom = buffer[5];
        _header.control_1 = buffer[6];
        _header.control_2 = buffer[7];
        _header.n_ram_banks = buffer[8];

        // Stores the prg_rom
        let mut _prg_rom = vec![];
        let mut buffer: [u8;16*1024] = [0;16*1024];
        for _i in 0.._header.n_prg_rom {
            match file.read(&mut buffer) {
                Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
                Ok(_) => ()
            }
            _prg_rom.push(buffer);
        }

        // Stores the chr_rom
        let mut _chr_rom = vec![];
        let mut buffer: [u8;8*1024] = [0;8*1024];
        for _i in 0.._header.n_chr_rom {
            match file.read(&mut buffer) {
                Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
                Ok(_) => ()
            }
            _chr_rom.push(buffer);
        }
        if _chr_rom.len() == 0 {
            _chr_rom.push(buffer);
        }

        Cartridge {
            header: _header,
            prg_rom: _prg_rom,
            chr_rom: _chr_rom
        }
    }
}