// Represents the CPU bus of the NES

// ===== IMPORTS =====

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::apu::Apu;
use crate::cartridge::mapper::Mapper;
use crate::controllers::Controller;
use crate::ppu::Ppu;
use crate::state::Stateful;

// ===== CONSTANTS =====

pub const STACK_OFFSET: u16 = 0x100;

// ===== TYPE ALIAS =====

type MapperRc = Rc<RefCell<Box<dyn Mapper>>>;

// ===== BUS STRUCT =====

pub struct Bus {
    cpu_ram: [u8; 0x0800],
    o_p_mapper: Option<MapperRc>,
    p_ppu: Rc<RefCell<Ppu>>,
    p_apu: Rc<RefCell<Apu>>,

    controllers: [Controller; 2],
}

impl Bus {
    pub fn new(p_ppu: Rc<RefCell<Ppu>>, p_apu: Rc<RefCell<Apu>>) -> Self {
        Bus {
            cpu_ram: [0; 0x0800],
            o_p_mapper: None,
            p_ppu,
            p_apu,

            controllers: [Controller::new(); 2],
        }
    }

    pub fn from_state(state: &BusState, p_ppu: Rc<RefCell<Ppu>>, p_apu: Rc<RefCell<Apu>>) -> Self {
        let mut bus = Bus::new(p_ppu, p_apu);
        bus.set_state(state);
        bus
    }

    pub fn set_mapper(&mut self, p_mapper: MapperRc) {
        self.o_p_mapper = Some(p_mapper);
    }

    pub fn get_scanline(&self) -> u16 {
        self.p_ppu.borrow().get_scanline()
    }

    pub fn get_cycles(&self) -> u16 {
        self.p_ppu.borrow().get_cycles()
    }

    pub fn set_input(&mut self, id: usize, input: u8) {
        self.controllers[id].buffer = input;
    }

    // Reads data from the bus at the specified address
    pub fn read(&mut self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => Ok(self.cpu_ram[address as usize]),
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => Ok(self.cpu_ram[(address & 0x07FF) as usize]),
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => match self.p_ppu.borrow_mut().read_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => match self.p_ppu.borrow_mut().read_register(address & 0x2007) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4000 - 0x4013 / NES APU I/O Registers
            0x4000..=0x4013 => match self.p_apu.borrow_mut().read_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4014 / NES PPU Register
            0x4014 => match self.p_ppu.borrow_mut().read_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4015 / NES APU Register
            0x4015 => match self.p_apu.borrow_mut().read_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4016 / First controller
            0x4016 => Ok(self.controllers[0].check_shifter()),
            // 0x4017 / Second controller
            0x4017 => Ok(self.controllers[1].check_shifter()),
            // 0x4018 - 0x4020 / I/O Refisters
            0x4018..=0x4020 => Ok(0),
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => {
                match self
                    .o_p_mapper
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .prg_rom_read(address)
                {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        debug!("{}", e);
                        Ok(0)
                    }
                }
            }
        }
    }

    // Used for debugging
    // Some normal reads may change the state of some elements (ex: 2002 for PPU)
    // Use this method to avoid it
    pub fn read_only(&self, address: u16) -> Result<u8, Box<dyn Error>> {
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => Ok(self.cpu_ram[address as usize]),
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => Ok(self.cpu_ram[(address & 0x07FF) as usize]),
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => match self.p_ppu.borrow().read_only_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => match self.p_ppu.borrow().read_only_register(address & 0x2007) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4000 - 0x4013 / NES APU I/O Registers
            0x4000..=0x4013 => match self.p_apu.borrow().read_only_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4014 / NES PPU Register
            0x4014 => match self.p_ppu.borrow().read_only_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4015 / NES APU Register
            0x4015 => match self.p_apu.borrow().read_only_register(address) {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!("{}", e);
                    Ok(0)
                }
            },
            // 0x4016 / First controller
            0x4016 => Ok(0),
            // 0x4017 / Second controller
            0x4017 => Ok(0),
            // 0x4018 - 0x4020 / I/O Refisters
            0x4018..=0x4020 => Ok(0),
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => {
                match self
                    .o_p_mapper
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .prg_rom_read(address)
                {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        debug!("{}", e);
                        Ok(0)
                    }
                }
            }
        }
    }

    // Writes data to the bus at the specified address
    pub fn write(&mut self, address: u16, value: u8) -> Result<(), Box<dyn Error>> {
        match address {
            // 0x0000 - 0x07FF / 2KB CPU RAM
            0x0000..=0x7FF => self.cpu_ram[address as usize] = value,
            // 0x0800 - 0x1FFF / CPU RAM Mirrors
            0x0800..=0x1FFF => self.cpu_ram[(address & 0x07FF) as usize] = value,
            // 0x2000 - 0x2007 / NES PPU Registers
            0x2000..=0x2007 => {
                if let Err(e) = self.p_ppu.borrow_mut().write_register(address, value) {
                    debug!("{}", e);
                }
            }
            // 0x2008 - 0x3FFF / NES PPU Registers Mirrors
            0x2008..=0x3FFF => {
                if let Err(e) = self
                    .p_ppu
                    .borrow_mut()
                    .write_register(address & 0x2007, value)
                {
                    debug!("{}", e);
                }
            }
            // 0x4000 - 0x4013 / NES APU I/O Registers
            0x4000..=0x4013 => {
                if let Err(e) = self.p_apu.borrow_mut().write_register(address, value) {
                    debug!("{}", e);
                }
            }
            // 0x4014 / NES PPU Register
            0x4014 => {
                if let Err(e) = self.p_ppu.borrow_mut().write_register(address, value) {
                    debug!("{}", e);
                }
            }
            // 0x4015 / NES APU Register
            0x4015 => {
                if let Err(e) = self.p_apu.borrow_mut().write_register(address, value) {
                    debug!("{}", e);
                }
            }
            // 0x4016 / First controller
            0x4016 => {
                if (value & 0x01) > 0 {
                    self.controllers[0].update_shifter();
                }
            }
            // 0x4017 / Second controller + NES APU Register
            0x4017 => {
                if let Err(e) = self.p_apu.borrow_mut().write_register(address, value) {
                    debug!("{}", e);
                }
                if (value & 0x01) > 0 {
                    self.controllers[1].update_shifter();
                }
            }
            // 0x4018 - 0x4020 / I/O Refisters
            0x4018..=0x4020 => (),
            // 0x4021 - 0xFFFF / Handled by the mapper
            0x4021..=0xFFFF => {
                if let Err(e) = self
                    .o_p_mapper
                    .as_mut()
                    .unwrap()
                    .borrow_mut()
                    .prg_rom_write(address, value)
                {
                    debug!("{}", e);
                }
            }
        }
        Ok(())
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BusState {
    #[serde_as(as = "[_; 0x0800]")]
    cpu_ram: [u8; 0x0800],
    controllers: [Controller; 2],
}

impl Stateful for Bus {
    type State = BusState;

    fn get_state(&self) -> Self::State {
        BusState {
            cpu_ram: self.cpu_ram,
            controllers: self.controllers,
        }
    }

    fn set_state(&mut self, state: &Self::State) {
        self.cpu_ram = state.cpu_ram;
        self.controllers = state.controllers;
    }
}
