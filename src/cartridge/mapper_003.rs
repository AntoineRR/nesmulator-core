// Mapper 3 : CNROM

use std::{any::Any, error::Error};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::mapper::{INesHeader, Mapper, Mirroring};
use crate::{
    cartridge::mapper::MapperState,
    errors::{InvalidMapperReadError, InvalidMapperWriteError},
    state::Stateful,
};

pub struct Mapper3 {
    header: INesHeader,
    selected_chr_rom: usize,
    prg_rom: Vec<[u8; 0x4000]>,
    chr_rom: Vec<[u8; 0x2000]>,
}

impl Mapper3 {
    pub fn new(
        prg_rom: Vec<[u8; 16 * 1024]>,
        chr_rom: Vec<[u8; 8 * 1024]>,
        header: INesHeader,
    ) -> Self {
        Mapper3 {
            header,
            selected_chr_rom: 0,
            prg_rom,
            chr_rom,
        }
    }
}

impl Mapper for Mapper3 {
    fn prg_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x0000..=0x401F => Err(Box::new(InvalidMapperReadError(address))),
            0x4020..=0x5FFF => Err(Box::new(InvalidMapperReadError(address))),
            0x6000..=0x7FFF => Err(Box::new(InvalidMapperReadError(address))),
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
            0x6000..=0x7FFF => Err(Box::new(InvalidMapperWriteError(address))),
            0x8000..=0xFFFF => {
                self.selected_chr_rom = (value & 0x03) as usize;
                Ok(())
            }
        }
    }

    fn chr_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        Ok(self.chr_rom[self.selected_chr_rom][address as usize])
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
        match state.as_any().downcast_ref::<Mapper3State>() {
            Some(s) => self.set_state(s),
            None => panic!("State is not a Mapper3State"),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Mapper3State {
    header: INesHeader,
    selected_chr_rom: usize,
    #[serde_as(as = "Vec<[_; 0x2000]>")]
    chr_rom: Vec<[u8; 0x2000]>,
}

#[typetag::serde]
impl MapperState for Mapper3State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Stateful for Mapper3 {
    type State = Mapper3State;

    fn get_state(&self) -> Self::State {
        Mapper3State {
            header: self.header.clone(),
            selected_chr_rom: self.selected_chr_rom,
            chr_rom: self.chr_rom.clone(),
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.header = state.header.clone();
        self.selected_chr_rom = state.selected_chr_rom;
        self.chr_rom = state.chr_rom.clone();
    }
}
