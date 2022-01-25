use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::Bus,
    cpu::{cpu::CPU, enums::Interrupt},
};

const DMC_RATE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct DMC {
    p_bus: Option<Rc<RefCell<Bus>>>,
    p_cpu: Option<Rc<RefCell<CPU>>>,

    pub interrupt_flag: bool,
    irq_enabled: bool,
    loop_flag: bool,
    rate: u16,

    sample_address: u16,
    sample_length: u16,
    sample_buffer: u8,
    current_address: u16,
    bytes_remaining: u16,

    silence_flag: bool,
    bits_remaining: u8,

    timer: u16,
    period: u16,

    output_level: u8,
}

impl DMC {
    pub fn new() -> Self {
        DMC {
            p_bus: None,
            p_cpu: None,

            interrupt_flag: false,
            irq_enabled: false,
            loop_flag: false,
            rate: 0,

            sample_address: 0,
            sample_length: 0,
            sample_buffer: 0,
            current_address: 0,
            bytes_remaining: 0,

            silence_flag: false,
            bits_remaining: 1,

            timer: 0,
            period: 0,

            output_level: 0,
        }
    }

    pub fn attach_bus_and_cpu(&mut self, p_bus: Rc<RefCell<Bus>>, p_cpu: Rc<RefCell<CPU>>) {
        self.p_bus = Some(p_bus);
        self.p_cpu = Some(p_cpu);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.silence_flag = enabled;
        self.interrupt_flag = false;
        if self.silence_flag {
            self.bytes_remaining = 0;
        } else {
            if self.bytes_remaining == 0 {
                self.current_address = self.sample_address;
                self.bytes_remaining = self.sample_length;
            }
        }
    }

    pub fn is_active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn set_rate(&mut self, value: u8) {
        self.irq_enabled = value & 0x80 > 0;
        if !self.irq_enabled {
            self.interrupt_flag = false;
        }
        self.loop_flag = value & 0x40 > 0;
        // Divide by 2 because the DMC_RATE table on the nesdev wiki contains the number of CPU cycles
        // and the DMC is clocked every 2 CPU cycles
        self.rate = DMC_RATE[(value & 0x0F) as usize] / 2;
    }

    pub fn set_output_level(&mut self, value: u8) {
        self.output_level = value & 0x7F;
    }

    pub fn set_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 | ((value as u16) << 6);
    }

    pub fn set_sample_length(&mut self, value: u8) {
        self.sample_length = ((value as u16) << 4) | 0x0001;
    }

    fn clock_reader(&mut self) {
        if self.bytes_remaining == 0 || self.bits_remaining != 0 {
            return;
        }

        if let Some(bus) = &self.p_bus {
            self.sample_buffer = bus.borrow_mut().read(self.current_address);
        } else {
            panic!("No bus attached to the DMC");
        }

        if self.current_address < 0xFFFF {
            self.current_address += 1;
        } else {
            self.current_address = 0x8000;
        }
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.current_address = self.sample_address;
                self.bytes_remaining = self.sample_length;
            } else if self.irq_enabled {
                self.interrupt_flag = true;
            }
        }
    }

    fn clock_output(&mut self) {
        if !self.silence_flag {
            if self.sample_buffer & 0x01 > 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else {
                if self.output_level >= 2 {
                    self.output_level -= 2;
                }
            }
            self.sample_buffer >>= 1;
        }
        self.bits_remaining -= 1;
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
        }
    }

    pub fn clock(&mut self) {
        if self.interrupt_flag {
            if let Some(cpu) = &self.p_cpu {
                cpu.borrow_mut().interrupt(Interrupt::IRQ);
            } else {
                panic!("No CPU set for the DMC");
            }
        }
        self.clock_reader();
        if self.timer != 0 {
            self.timer -= 1;
        } else {
            self.timer = self.period + 1;
            self.clock_output();
        }
    }

    pub fn get_output(&self) -> u8 {
        self.output_level
    }
}
