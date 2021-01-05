// Represents the PPU of the NES i.e. a component
// with a similar behaviour as the 2C02

// ===== IMPORTS =====

use std::sync::{Arc, Mutex};
use crate::gui::GUI;

// ===== STRUCT =====

#[derive(Debug)]
pub struct PPU {
    // PPU registers
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_addr: u8,
    pub oam_data: u8,
    pub scroll: u8,
    pub addr: u8,
    pub data: u8,
    pub oam_dma: u8,

    // Variables required for display
    pub cycles: u16,
    pub scanline: u16,
    pub buffer: [u32;256*341],

    // GUI
    pub p_gui: Arc<Mutex<GUI>>
}

impl PPU {
    pub fn new(p_gui: Arc<Mutex<GUI>>) -> Self {
        PPU {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
            oam_dma: 0,
            cycles: 0,
            scanline: 0,
            buffer: [0;256*341],
            p_gui
        }
    }

    // ===== CLOCK =====

    // Executes a clock cycle
    pub fn clock(&mut self) {
        // Set the color of one pixel
        let white: u32 = 0b00000000_11111111_11111111_11111111;
        let black: u32 = 0b00000000_00000000_00000000_00000000;
        self.buffer[(256*self.scanline as u32 + self.cycles as u32) as usize] = match rand::random() {
            true => white,
            false => black
        };
        if self.cycles >= 341 {
            self.scanline += 1;
            self.cycles = 0;
            if self.scanline >= 261 {
                self.scanline = 0;
                self.p_gui.lock().unwrap().update(self.buffer.to_vec());
            }
        }
        self.cycles += 1;
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => self.ctrl = value,
            0x2001 => self.mask = value,
            0x2002 => self.status = value,
            0x2003 => self.oam_addr = value,
            0x2004 => self.oam_data = value,
            0x2005 => self.scroll = value,
            0x2006 => self.addr = value,
            0x2007 => self.data = value,
            0x4014 => self.oam_dma = value,
            _ => panic!("Wrong address given to PPU : {:#x}",address)
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => self.ctrl,
            0x2001 => self.mask,
            0x2002 => self.status,
            0x2003 => self.oam_addr,
            0x2004 => self.oam_data,
            0x2005 => self.scroll,
            0x2006 => self.addr,
            0x2007 => self.data,
            0x4014 => self.oam_dma,
            _ => panic!("Wrong address given to PPU : {:#x}",address)
        }
    }
}