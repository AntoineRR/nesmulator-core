// Mapper 2 : UNROM

use std::{any::Any, error::Error};

use serde::{Deserialize, Serialize};

use super::mapper::{INesHeader, Mapper, MapperState, Mirroring};
use crate::{
    errors::{InvalidMapperReadError, InvalidMapperWriteError},
    state::Stateful,
};

pub struct Mapper2 {
    header: INesHeader,
    lo_prg_rom: usize,
    prg_rom: Vec<[u8; 0x4000]>,
    chr_rom: Vec<[u8; 0x2000]>,
}

impl Mapper2 {
    pub fn new(prg_rom: Vec<[u8; 0x4000]>, chr_rom: Vec<[u8; 0x2000]>, header: INesHeader) -> Self {
        Mapper2 {
            header,
            lo_prg_rom: 0,
            prg_rom,
            chr_rom,
        }
    }
}

impl Mapper for Mapper2 {
    fn prg_rom_read(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            0x0000..=0x401F => Err(Box::new(InvalidMapperReadError(address))),
            0x4020..=0x5FFF => Err(Box::new(InvalidMapperReadError(address))),
            0x6000..=0x7FFF => Err(Box::new(InvalidMapperReadError(address))),
            0x8000..=0xBFFF => Ok(self.prg_rom[self.lo_prg_rom][(address & 0x3FFF) as usize]),
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
                self.lo_prg_rom = (value & 0x0F) as usize;
                Ok(())
            }
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

    fn set_mapper_state(&mut self, state: &Box<dyn MapperState>) {
        match state.as_any().downcast_ref::<Mapper2State>() {
            Some(s) => self.set_state(s),
            None => panic!("State is not a Mapper2State"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mapper2State {
    header: INesHeader,
    lo_prg_rom: usize,
}

#[typetag::serde]
impl MapperState for Mapper2State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Stateful for Mapper2 {
    type State = Mapper2State;

    fn get_state(&self) -> Self::State {
        Mapper2State {
            header: self.header.clone(),
            lo_prg_rom: self.lo_prg_rom,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.header = state.header.clone();
        self.lo_prg_rom = state.lo_prg_rom;
    }
}
