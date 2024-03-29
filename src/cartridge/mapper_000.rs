// Mapper 0 : NROM

use std::{any::Any, error::Error};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::mapper::{INesHeader, Mapper, MapperState, Mirroring};
use crate::{
    errors::{InvalidMapperReadError, InvalidMapperWriteError},
    state::Stateful,
};

pub struct Mapper0 {
    header: INesHeader,
    prg_rom: Vec<[u8; 0x4000]>,
    chr_rom: Vec<[u8; 0x2000]>,
    ram: [u8; 0x2000], // There can be RAM on Family Basic ROMs
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<[u8; 0x4000]>, chr_rom: Vec<[u8; 0x2000]>, header: INesHeader) -> Self {
        Mapper0 {
            header,
            prg_rom,
            chr_rom,
            ram: [0; 0x2000],
        }
    }
}

impl Mapper for Mapper0 {
    fn prg_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x0000..=0x401F => Err(Box::new(InvalidMapperReadError(address))),
            0x4020..=0x5FFF => Err(Box::new(InvalidMapperReadError(address))),
            0x6000..=0x7FFF => Ok(self.ram[(address & 0x1FFF) as usize]),
            0x8000..=0xBFFF => Ok(self.prg_rom[0][(address & 0x3FFF) as usize]),
            0xC000..=0xFFFF => {
                Ok(self.prg_rom[(self.prg_rom.len() - 1) as usize][(address & 0x3FFF) as usize])
            }
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
            0x8000..=0xFFFF => Err(Box::new(InvalidMapperWriteError(address))),
        }
    }

    fn chr_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        Ok(self.chr_rom[0][address as usize])
    }

    fn chr_rom_write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match address {
            0x0000..=0x1FFF => {
                self.chr_rom[0][address as usize] = value;
                Ok(())
            }
            _ => Err(Box::new(InvalidMapperWriteError(address))),
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.header.mirroring
    }

    fn get_mapper_state(&self) -> Box<dyn MapperState> {
        Box::new(self.get_state())
    }

    fn set_mapper_state(&mut self, state: &dyn MapperState) {
        match state.as_any().downcast_ref::<Mapper0State>() {
            Some(s) => self.set_state(s),
            None => panic!("State is not a Mapper0State"),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Mapper0State {
    header: INesHeader,
    #[serde_as(as = "[_; 0x2000]")]
    ram: [u8; 0x2000],
    #[serde_as(as = "Vec<[_; 0x2000]>")]
    chr_rom: Vec<[u8; 0x2000]>,
}

#[typetag::serde]
impl MapperState for Mapper0State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Stateful for Mapper0 {
    type State = Mapper0State;

    fn get_state(&self) -> Self::State {
        Mapper0State {
            header: self.header.clone(),
            ram: self.ram,
            chr_rom: self.chr_rom.clone(),
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.header = state.header.clone();
        self.ram = state.ram;
        self.chr_rom = state.chr_rom.clone();
    }
}
