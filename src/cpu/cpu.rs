// Implements the CPU of the NES, i.e. a component
// with a similar behavior as the rp2A03 / 6502

// ====== IMPORTS =====

use core::panic;
use std::sync::{Arc, Mutex};

use crate::{bus::Bus};
use crate::bus::STACK_OFFSET;
use super::instructions::{CpuInstruction,INSTRUCTIONS};
use super::enums::{AdressingMode as am,Flag,Interrupt};
use std::fmt::Write;

// ===== CPU STRUCT =====

// This struct contains the various registers of the CPU
#[derive(Debug)]
pub struct CPU {
    // Registers
    pub a: u8, // accumulator
    pub x: u8, // x index
    pub y: u8, // y index
    pub pc: u16, // program counter
    pub sp: u8, // stack pointer
    pub p: u8, // status flags

    // Cycles required by the instruction to complete
    pub cycles: u8,

    // Does the current instruction require an eventual additional cycle ?
    pub require_add_cycle: bool,

    // Total clock cycles from the start of the CPU
    pub total_clock: u64,

    // pointer to the data bus where we read from and write to
    pub p_bus: Arc<Mutex<Bus>>
}

impl CPU {
    pub fn new(p_bus: Arc<Mutex<Bus>>) -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,
            p: 0x34,

            cycles: 0,

            require_add_cycle: false,

            total_clock: 0,

            p_bus
        }
    }

    // ===== BUS ACCESS =====

    // Reads data from the bus at the given address
    pub fn read_bus(& self, address: u16) -> u8 {
        self.p_bus.lock().unwrap().read(address)
    }

    // Writes data to the bus at the given address
    pub fn write_bus(&mut self, address: u16, data: u8) {
        self.p_bus.lock().unwrap().write(address, data);
    }

    // Pushes data to stack
    pub fn push_to_stack(&mut self, data: u8) {
        self.write_bus(STACK_OFFSET + self.sp as u16, data);
        if self.sp != 0 {
            self.sp -= 1;
        }
        else {
            self.sp = 255;
        }
    }

    // Returns data from the stack
    pub fn pop_from_stack(&mut self) -> u8 {
        if self.sp != 255 {
            self.sp += 1;
        }
        else {
            self.sp = 0;
        }
        self.read_bus(STACK_OFFSET + self.sp as u16)
    }

    // ===== FLAG SETTER AND GETTER =====

    // Returns the required flag from the status register
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.p & flag as u8 != 0
    }

    // Toggles the required flag in the status register
    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.p |= flag as u8;
        }
        else {
            self.p &= !(flag as u8);
        }
    }

    // ===== INTERRUPTS =====

    // Called when an interrupt occurs
    // The interrupt disable flag from the status register needs to be 0
    pub fn interrupt(&mut self, interrupt_type: Interrupt) {
        if !self.get_flag(Flag::InterruptDisable) || interrupt_type == Interrupt::NMI {

            if interrupt_type != Interrupt::Reset {
                // Push program counter and status register on the stack
                self.push_to_stack(((self.pc & 0xFF00) >> 8) as u8);
                self.push_to_stack((self.pc & 0x00FF) as u8);
                self.push_to_stack(self.p);
            }
            // Disable interrupts
            self.set_flag(Flag::InterruptDisable, true);
            // Load interrupt handler address into the program counter
            let start_address: u16;
            match interrupt_type {
                Interrupt::IRQ => start_address = 0xFFFE,
                Interrupt::NMI => start_address = 0xFFFA,
                Interrupt::Reset => start_address = 0xFFFC
            }
            self.pc = self.read_bus(start_address) as u16 + ((self.read_bus(start_address + 1) as u16) << 8) as u16;

            if interrupt_type == Interrupt::NMI {
                self.cycles = 7;
            }
            else {
                self.cycles = 7;
            }
        }
    }

    // Called when the reset button is pressed on the NES
    pub fn reset(&mut self) {
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.sp = 0xFD;
        self.p = 0x20;
        self.interrupt(Interrupt::Reset);
    }

    // ===== CLOCK =====

    // Executes a clock cycle
    // opcodes for operations are stored in the INSTRUCTIONS const
    pub fn clock(&mut self) {
        // Basic cycle emulation :
        // cycle 0 does the operation and the others do nothing
        if self.cycles == 0 {
            // Get operation code
            let opcode: u8 = self.read_bus(self.pc);
            //self.display_cpu_log(opcode); // Uncomment for debugging
            
            // Get instruction information for the operation code
            let instruction: &CpuInstruction = &INSTRUCTIONS[opcode as usize];
            self.require_add_cycle = instruction.add_cycle;
            
            // Execute the instruction
            (instruction.execute)(self, instruction.adressing_mode);
            
            // Increase program counter
            self.pc += 1;
            
            // Sets the correct number of cycles
            self.cycles += instruction.cycles - 1;
        }
        else {
            self.cycles -= 1;
        }
        self.total_clock += 1;
    }

    // ===== ADDRESSING MODES =====

    // Returns the parameters for the instruction as an address
    pub fn fetch_address(&mut self, mode: am) -> u16 {
        match mode {
            am::Implicit => 0,
            am::Accumulator => {
                self.a as u16
            },
            am::Immediate => {
                self.pc += 1;
                self.pc
            },
            am::ZeroPage => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                address as u16
            },
            am::ZeroPageX => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                (address as u16 + self.x as u16) % 0x100
            },
            am::ZeroPageY => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                (address as u16 + self.y as u16) % 0x100
            },
            am::Relative => {
                self.pc += 1;
                self.pc
            },
            am::Absolute => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                lo as u16 + ((hi as u16) << 8)
            },
            am::AbsoluteX => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let address: u16 = lo as u16 + ((hi as u16) << 8);
                let result: u16 = address + self.x as u16;
                if (self.require_add_cycle) && ((result & 0xFF00) != (address & 0xFF00)) {
                    self.cycles += 1;
                }
                result
            },
            am::AbsoluteY => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let address: u16 = lo as u16 + ((hi as u16) << 8);
                let addition: u32 = address as u32 + self.y as u32;
                let result: u16 = (addition & 0x0000_FFFF) as u16;
                if (self.require_add_cycle) && ((result & 0xFF00) != (address & 0xFF00)) {
                    self.cycles += 1;
                }
                result
            },
            am::Indirect => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let ptr: u16 = lo as u16 + ((hi as u16) << 8);
                let address_lo: u8;
                let address_hi: u8;
                if lo == 0xFF {
                    // Hardware bug
                    address_lo = self.read_bus(ptr);
                    address_hi = self.read_bus(ptr & 0xFF00);
                }
                else {
                    address_lo = self.read_bus(ptr);
                    address_hi = self.read_bus(ptr + 1);
                }
                address_lo as u16 + ((address_hi as u16) << 8)
            },
            am::IndirectX => {
                self.pc += 1;
                let ptr_lo: u16 = (self.read_bus(self.pc) as u16 + self.x as u16) % 0x100; // address in the 0x00 page
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                self.read_bus(ptr_lo) as u16 + ((self.read_bus(ptr_hi) as u16) << 8)
            },
            am::IndirectY => {
                self.pc += 1;
                let ptr_lo: u16 = (self.read_bus(self.pc) as u16) % 0x100; // address in the 0x00 page
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address_lo: u8 = self.read_bus(ptr_lo);
                let address_hi: u8 = self.read_bus(ptr_hi);
                let address: u16 = address_lo as u16 + ((address_hi as u16) << 8);
                let result: u16 = ((address as u32 + self.y as u32) % 0x10000) as u16;
                if (self.require_add_cycle) && ((result & 0xFF00) != (address & 0xFF00)) {
                    self.cycles += 1;
                }
                result
            },
            am::NoMode => {
                panic!("No mode specified when trying to fetch data !");
            }
        }
    }

    // ===== INSTRUCTIONS =====

    // Add with carry
    // A,Z,C,N = A+M+C
    pub fn adc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u16 = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        self.set_flag(Flag::Overflow, !(previous_a ^ data) & (previous_a ^ (result as u8)) == 0x80);
    }

    // Logical and
    // A,Z,N = A & M
    pub fn and(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & 0x80 == 0x80);
    }

    // Arithmetic shift left
    // A,Z,C,N = M*2 or M,Z,C,N = M*2
    pub fn asl(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let result: u16;
        match mode {
            am::Accumulator => {
                result = (self.a as u16) << 1;
                self.a = result as u8;
            }
            _ => {
                result = data << 1;
                self.write_bus(address, result as u8);
            }
        }
        self.set_flag(Flag::Carry, (result & 0xFF00) > 0);
        self.set_flag(Flag::Zero, (result & 0x00FF) == 0);
        self.set_flag(Flag::Negative, result & 0x0080 == 0x0080);
    }

    // Branch if carry clear
    // (C = 0) => pc += addr
    pub fn bcc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Carry) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Branch if carry set
    // (C = 1) => pc += addr
    pub fn bcs(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if self.get_flag(Flag::Carry) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Branch if equal
    // (Z = 1) => pc += addr
    pub fn beq(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if self.get_flag(Flag::Zero) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Bit test
    // A & M, N = M7, V = M6
    pub fn bit(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = self.a & data;
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (data & 0b1000_0000) == 0b1000_0000);
        self.set_flag(Flag::Overflow, (data & 0b0100_0000) == 0b0100_0000);
    }

    // Branch if minus
    // (N = 1) => pc += addr
    pub fn bmi(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if self.get_flag(Flag::Negative) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Branch if not equal
    // (Z = 0) => pc += addr
    pub fn bne(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Zero) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Branch if positive
    // (N = 0) => pc += addr
    pub fn bpl(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Negative) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != ((self.pc + 1) & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Force interrupt
    pub fn brk(&mut self, _: am) {
        self.pc += 1;
        self.set_flag(Flag::Break, true);
        self.interrupt(Interrupt::IRQ);
    }

    // Branch if overflow clear
    // (V = 0) => pc += addr
    pub fn bvc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Overflow) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Branch if overflow set
    // (V = 1) => pc += addr
    pub fn bvs(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if self.get_flag(Flag::Overflow) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if ((result + 1) as u16 & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Clear carry flag
    // C = 0
    pub fn clc(&mut self, _: am) {
        self.set_flag(Flag::Carry, false);
    }

    // Clear decimal mode
    // D = 0
    pub fn cld(&mut self, _: am) {
        self.set_flag(Flag::Decimal, false);
    }

    // Clear interrupt disable
    // I = 0
    pub fn cli(&mut self, _: am) {
        self.set_flag(Flag::InterruptDisable, false);
    }

    // Clear overflow flag
    // V = 0
    pub fn clv(&mut self, _: am) {
        self.set_flag(Flag::Overflow, false);
    }

    // Compare
    // Z,C,N = A-M
    pub fn cmp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        if self.a > data {
            result = self.a - data;
        }
        else {
            result = data - self.a;
        }
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, self.a >= data);
        self.set_flag(Flag::Negative, (data > self.a) | ((data == 0) & ((self.a & 0x80) == 0x80)));
    }

    // Compare x register
    // Z,C,N = X-M
    pub fn cpx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        if self.x > data {
            result = self.x - data;
        }
        else {
            result = data - self.x;
        }
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, self.x >= data);
        self.set_flag(Flag::Negative, (data > self.x) | ((data == 0) & ((self.x & 0x80) == 0x80)));
    }

    // Compare y register
    // Z,C,N = Y-M
    pub fn cpy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        if self.y > data {
            result = self.y - data;
        }
        else {
            result = data - self.y;
        }
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, self.y >= data);
        self.set_flag(Flag::Negative, (data > self.y) | ((data == 0) & ((self.y & 0x80) == 0x80)));
    }

    // Decrement memory
    // M,Z,N = M-1
    pub fn dec(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        if data != 0 {
            result = data - 1;
        }
        else {
            result = 255;
        }
        self.write_bus(address, result);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Decrement x register
    // X,Z,N = X-1
    pub fn dex(&mut self, _: am) {
        if self.x != 0 {
            self.x -= 1;
        }
        else {
            self.x = 255;
        }
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Decrement y register
    // Y,Z,N = Y-1
    pub fn dey(&mut self, _: am) {
        if self.y != 0 {
            self.y -= 1;
        }
        else {
            self.y = 255;
        }
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) == 0x80);
    }

    // Exclusive or
    // A,Z,N = A^M
    pub fn eor(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a ^= data;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Increment memory
    // M,Z,N = M+1
    pub fn inc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        if data != 255 {
            result = data + 1;
        }
        else {
            result = 0;
        }
        self.write_bus(address, result);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Increment x register
    // X,Z,N = X+1
    pub fn inx(&mut self, _: am) {
        if self.x != 255 {
            self.x += 1;
        }
        else {
            self.x = 0;
        }
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Increment y register
    // Y,Z,N = Y+1
    pub fn iny(&mut self, _: am) {
        if self.y != 255 {
            self.y += 1;
        }
        else {
            self.y = 0;
        }
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) == 0x80);
    }

    // Jump
    // pc = addr
    pub fn jmp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.pc = address - 1;
    }

    // Jump to subroutine
    pub fn jsr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.push_to_stack((self.pc >> 8) as u8);
        self.push_to_stack(self.pc as u8);
        self.pc = address - 1;
    }

    // Load accumulator
    // A,Z,N = M
    pub fn lda(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a = data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Load x register
    // X,Z,N = M
    pub fn ldx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.x = data;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Load y register
    // Y,Z,N = M
    pub fn ldy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.y = data;
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) == 0x80);
    }

    // Logical shift right
    // A,C,Z,N = A/2 or M,C,Z,N = M/2
    pub fn lsr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let result: u8;
        let previous: u8;
        match mode {
            am::Accumulator => {
                previous = self.a;
                result = self.a >> 1;
                self.a = result;
            }
            _ => {
                previous = data as u8;
                result = ((data >> 1) & 0x00FF) as u8;
                self.write_bus(address, result);
            }
        }
        self.set_flag(Flag::Carry, (previous & 0x01) > 0);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, result & 0x80 == 0x80);
    }

    // No operation
    pub fn nop(&mut self, mode: am) {
        self.fetch_address(mode);
    }

    // Logical inclusive or
    // A,Z,N = A|M
    pub fn ora(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a |= data;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, self.a & 0x80 == 0x80);
    }

    // Push accumulator
    // A => stack
    pub fn pha(&mut self, _: am) {
        self.push_to_stack(self.a);
    }

    // Push processor status
    // status => stack
    pub fn php(&mut self, _: am) {
        self.push_to_stack(self.p | Flag::Break as u8 | Flag::Unused as u8);
    }

    // Pull accumulator
    // A <= stack
    pub fn pla(&mut self, _: am) {
        self.a = self.pop_from_stack();
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, self.a & 0x80 == 0x80);
    }

    // Pull processor status
    // status <= stack
    pub fn plp(&mut self, _: am) {
        let status: u8 = self.pop_from_stack();
        self.set_flag(Flag::Carry, status & (Flag::Carry as u8) == Flag::Carry as u8);
        self.set_flag(Flag::Zero, status & (Flag::Zero as u8) == Flag::Zero as u8);
        self.set_flag(Flag::InterruptDisable, status & (Flag::InterruptDisable as u8) == Flag::InterruptDisable as u8);
        self.set_flag(Flag::Decimal, status & (Flag::Decimal as u8) == Flag::Decimal as u8);
        self.set_flag(Flag::Unused, true);
        self.set_flag(Flag::Overflow, status & (Flag::Overflow as u8) == Flag::Overflow as u8);
        self.set_flag(Flag::Negative, status & (Flag::Negative as u8) == Flag::Negative as u8);
    }

    // Rotate left
    pub fn rol(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        let previous_value: u8;
        match mode {
            am::Accumulator => {
                previous_value = self.a;
                result = (self.a << 1) + (self.get_flag(Flag::Carry) as u8);
                self.a = result;
            }
            _ => {
                previous_value = data;
                result = (data << 1) + (self.get_flag(Flag::Carry) as u8);
                self.write_bus(address, result);
            }
        }
        self.set_flag(Flag::Carry, (previous_value & 0x80) == 0x80);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Rotate right
    pub fn ror(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8;
        let previous_value: u8;
        match mode {
            am::Accumulator => {
                previous_value = self.a;
                result = (self.a >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
                self.a = result;
            }
            _ => {
                previous_value = data;
                result = (data >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
                self.write_bus(address, result);
            }
        }
        self.set_flag(Flag::Carry, (previous_value & 0x01) == 0x01);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Return from interrupt
    // status <= stack, pc <= stack
    pub fn rti(&mut self, _: am) {
        let status: u8 = self.pop_from_stack();
        self.set_flag(Flag::Carry, status & (Flag::Carry as u8) == Flag::Carry as u8);
        self.set_flag(Flag::Zero, status & (Flag::Zero as u8) == Flag::Zero as u8);
        self.set_flag(Flag::InterruptDisable, status & (Flag::InterruptDisable as u8) == Flag::InterruptDisable as u8);
        self.set_flag(Flag::Decimal, status & (Flag::Decimal as u8) == Flag::Decimal as u8);
        self.set_flag(Flag::Unused, true);
        self.set_flag(Flag::Overflow, status & (Flag::Overflow as u8) == Flag::Overflow as u8);
        self.set_flag(Flag::Negative, status & (Flag::Negative as u8) == Flag::Negative as u8);
        let address: u16 = self.pop_from_stack() as u16 + ((self.pop_from_stack() as u16) << 8);
        self.pc = address - 1;
    }

    // Return from subroutine
    // pc - 1 <= stack
    pub fn rts(&mut self, _: am) {
        let address: u16 = self.pop_from_stack() as u16 + ((self.pop_from_stack() as u16) << 8);
        self.pc = address;
    }

    // Substract with carry
    // A,Z,C,N = A-M-(1-C)
    pub fn sbc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let original_data: u8 = self.read_bus(address);
        let data: u8 = original_data ^ 0xFF; // Converts data into a negative value + 1
        let result: u16 = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        // !!((A ^ Val) & (A ^ result) & 0x80) Only this, taken from Nintendulator could pass the nestest
        self.set_flag(Flag::Overflow, !!((previous_a ^ original_data) & (previous_a ^ (result as u8)) & 0x80) == 0x80);
    }

    // Set carry flag
    // C = 1
    pub fn sec(&mut self, _: am) {
        self.set_flag(Flag::Carry, true);
    }

    // Set decimal flag
    // D = 1
    pub fn sed(&mut self, _: am) {
        self.set_flag(Flag::Decimal, true);
    }

    // Set interrupt disable
    // I = 1
    pub fn sei(&mut self, _: am) {
        self.set_flag(Flag::InterruptDisable, true);
    }

    // Store accumulator
    // M = A
    pub fn sta(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.write_bus(address, self.a);
    }

    // Store x register
    // M = X
    pub fn stx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.write_bus(address, self.x);
    }

    // Store y register
    // M = Y
    pub fn sty(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.write_bus(address, self.y);
    }

    // Transfer accumulator to x
    // X = A
    pub fn tax(&mut self, _: am) {
        self.x = self.a;
        self.set_flag(Flag::Zero, self.x == 0x00);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Transfer accumulator to y
    // Y = A
    pub fn tay(&mut self, _: am) {
        self.y = self.a;
        self.set_flag(Flag::Zero, self.y == 0x00);
        self.set_flag(Flag::Negative, (self.y & 0x80) == 0x80);
    }

    // Transfer stack pointer to x
    // X = S
    pub fn tsx(&mut self, _: am) {
        self.x = self.sp;
        self.set_flag(Flag::Zero, self.x == 0x00);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Transfer x to accumulator
    // A = X
    pub fn txa(&mut self, _: am) {
        self.a = self.x;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Transfer x to stack pointer
    // S = X
    pub fn txs(&mut self, _: am) {
        self.sp = self.x;
    }

    // Transfer y to accumulator
    // A = Y
    pub fn tya(&mut self, _: am) {
        self.a = self.y;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // ===== UNDOCUMENTED OPCODES =====

    // Same as AND, with C flag
    // A,C,Z,N = A & M
    pub fn anc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & 0x80 == 0x80);
        self.set_flag(Flag::Carry, self.a & 0x80 == 0x80);
    }

    // Same as AND, with x transfered to a
    // A = X
    // A,Z,N = A & M
    pub fn ane(&mut self, mode: am) {
        self.a = self.x;
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & 0x80 == 0x80);
    }

    // Same as AND + ROR
    // C = bit 6
    // V = bit 5 != bit 6
    pub fn arr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.a = (self.a >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        self.set_flag(Flag::Carry, (self.a & 0x40) == 0x40); // bit 6 == 1
        self.set_flag(Flag::Overflow, ((self.a & 0x20) == 0x20) as u8 != ((self.a & 0x40) == 0x40) as u8); // bit 5 != 6
    }

    // Same as AND + shift right
    pub fn asr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let tmp: u8 = self.a & data;
        self.a = (tmp >> 1) as u8;
        self.set_flag(Flag::Carry, (tmp & 0x01) > 0);
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Same as DEC + CMP
    // M,C,Z,N = M-1
    pub fn dcp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let mut data: u8 = self.read_bus(address);
        if data != 0 {
            data -= 1;
        }
        else {
            data = 255;
        }
        self.write_bus(address, data);
        let result: u8;
        if self.a > data {
            result = self.a - data;
        }
        else {
            result = data - self.a;
        }
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, self.a >= data);
        self.set_flag(Flag::Negative, (self.a > data) || ((self.a == 0) && ((data & 0x80) == 0x80)));
    }

    // Same as INC + SBC
    // M = M+1
    // A,Z,C,N,V = A-M-(1-C)
    pub fn isb(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let original_data: u8 = self.read_bus(address);
        let inc_data: u8;
        if original_data != 255 {
            inc_data = original_data + 1;
        }
        else {
            inc_data = 0;
        }
        self.write_bus(address, inc_data);
        let data: u8 = inc_data ^ 0xFF; // Converts data into a negative value + 1
        let result: u16 = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        self.set_flag(Flag::Overflow, !!((previous_a ^ inc_data) & (previous_a ^ (result as u8)) & 0x80) == 0x80);
    }

    // Same as AND between M and SP
    // SP,A,X,N,Z = SP & M
    pub fn las(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let tmp: u8 = self.sp & data;
        self.sp = tmp;
        self.a = tmp;
        self.x = tmp;
        self.set_flag(Flag::Zero, tmp == 0x00);
        self.set_flag(Flag::Negative, (tmp & 0x80) == 0x80);
    }

    // Same as LDA + LDX
    // A,X,N,Z = M
    pub fn lax(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a = data;
        self.x = data;
        self.set_flag(Flag::Zero, data == 0x00);
        self.set_flag(Flag::Negative, (data & 0x80) == 0x80);
    }

    // Same as ORA #$EE + AND + TXA
    pub fn lxa(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a |= self.read_bus(0x00EE);
        self.a &= data;
        self.x = self.a;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Same as ROL + AND
    pub fn rla(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = (data << 1) + (self.get_flag(Flag::Carry) as u8);
        self.write_bus(address, result);
        self.a &= result;
        self.set_flag(Flag::Carry, (data & 0x80) == 0x80);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Same as ROR + ADC
    pub fn rra(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let ror_data: u8 = (data >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
        self.write_bus(address, ror_data);
        let carry: u8 = data & 0x01;
        let result: u16 = self.a as u16 + ror_data as u16 + carry as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        self.set_flag(Flag::Overflow, !(previous_a ^ ror_data) & (previous_a ^ (result as u8)) == 0x80);
    }

    // M = A & X
    pub fn sax(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let result: u8 = self.a & self.x;
        self.write_bus(address, result);
    }

    // X = (A&X)-M
    pub fn sbx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let mut data: u8 = (self.read_bus(address)) ^ 0xFF;
        if data < 255 {
            data += 1;
        }
        else {
            data = 0;
        }
        let result: u16 = ((self.x as u16) & (self.a as u16)) + (data as u16);
        self.x = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // M = A&X&(h[M]+1)
    pub fn sha(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let result: u8 = self.a & self.x & (((address & 0xFF00) >> 8) + 1) as u8;
        self.write_bus(address, result);
    }

    // SP = A&X
    // M = A&X&(h[M]+1)
    pub fn shs(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.sp = self.a & self.x;
        let result: u8 = self.a & self.x & (((address & 0xFF00) >> 8) + 1) as u8;
        self.write_bus(address, result);
    }

    // M = X&(h[M]+1)
    pub fn shx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let result: u8 = self.x & (((address & 0xFF00) >> 8) + 1) as u8;
        self.write_bus(address, result);
    }

    // M = Y&(h[M]+1)
    pub fn shy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let result: u8 = self.y & (((address & 0xFF00) >> 8) + 1) as u8;
        self.write_bus(address, result);
    }

    // Same as ASL + ORA
    pub fn slo(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let result: u16;
        result = (data as u16) << 1;
        self.write_bus(address, result as u8);
        self.a |= result as u8;
        self.set_flag(Flag::Carry, (result & 0xFF00) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Same as LSR + EOR
    pub fn sre(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let result: u8 = ((data >> 1) & 0x00FF) as u8;
        self.write_bus(address, result);
        self.a ^= result;
        self.set_flag(Flag::Carry, (data & 0x01) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
    }

    // Used for unvalid operation codes
    pub fn err(&mut self, _: am) {
        panic!("Encountered an unvalid opcode !");
    }

    // ===== DEBUGGING =====

    #[allow(dead_code)]
    pub fn display_cpu_log(&self, opcode: u8) {
        let mut instruction_and_parameters_str: String = String::from(format!("{:02X} ",opcode));
        let mut instruction_parameters: Vec<u8> = vec![];
        for i in 0..INSTRUCTIONS[opcode as usize].bytes - 1 {
            instruction_parameters.push(self.read_only_bus(self.pc + i as u16 + 1));
            match write!(instruction_and_parameters_str, "{:02X} ", instruction_parameters[i as usize]) {
                Err(why) => panic!("Error during write : {}",why),
                Ok(_) => ()
            }
        }
        while instruction_and_parameters_str.len() < 9 {
            instruction_and_parameters_str.push_str(" ");
        }
        let cpu_log: String = String::from(format!("{:04X}  {} {}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}"
            ,self.pc,instruction_and_parameters_str,self.dissassemble(opcode, instruction_parameters),
            self.a,self.x,self.y,self.p,self.sp));
        
        let scanline: u16 = self.p_bus.lock().unwrap().p_ppu.lock().unwrap().scanline;
        let mut scanline_str: String = scanline.to_string();
        while scanline_str.len() < 3 {
            scanline_str = String::from(format!(" {}",scanline_str));
        }
        let cycle: u16 = self.p_bus.lock().unwrap().p_ppu.lock().unwrap().cycles;
        let mut cycle_str: String = cycle.to_string();
        while cycle_str.len() < 3 {
            cycle_str = String::from(format!(" {}",cycle_str));
        }
        let ppu_log: String = String::from(format!("PPU:{},{}",scanline_str,cycle_str));

        println!("{} {} CYC:{}",cpu_log,ppu_log,self.total_clock);
    }

    #[allow(dead_code)]
    pub fn read_only_bus(&self, address: u16) -> u8 {
        self.p_bus.lock().unwrap().read_only(address)
    }

    #[allow(dead_code)]
    pub fn dissassemble(&self, opcode: u8, parameters: Vec<u8>) -> String {
        let mut dissassembly: String = String::from(INSTRUCTIONS[opcode as usize].name);
        dissassembly.push_str(" ");
        match INSTRUCTIONS[opcode as usize].adressing_mode {
            am::Accumulator => {
                if (opcode != 0xAA) && (opcode != 0x8A) {
                    dissassembly.push_str("A");
                }
            }
            am::Implicit => (),
            am::Immediate => dissassembly.push_str(&format!("#${:02X}",parameters[0])),
            am::ZeroPage => {
                let value: u8 = self.read_only_bus(parameters[0] as u16);
                dissassembly.push_str(&format!("${:02X} = {:02X}",parameters[0],value));
            },
            am::ZeroPageX => {
                let address: u16 = (parameters[0] as u16 + self.x as u16) % 0x100;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("${:02X},X @ {:02X} = {:02X}",parameters[0],address,value));
            },
            am::ZeroPageY => {
                let address: u16 = (parameters[0] as u16 + self.y as u16) % 0x100;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("${:02X},Y @ {:02X} = {:02X}",parameters[0],address,value));
            },
            am::Relative => {
                dissassembly.push_str(&format!("${:04X}",(self.pc as i16) + 2 + ((parameters[0] as i8) as i16)));
            },
            am::Absolute => {
                let address: u16 = parameters[0] as u16 + ((parameters[1] as u16) << 8) as u16;
                // Don't print value if it's a JMP / JSR
                if (opcode == 0x4C) || (opcode == 0x20) {
                    dissassembly.push_str(&format!("${:02X}{:02X}",parameters[1],parameters[0]));
                }
                else {
                    let value: u8 = self.read_only_bus(address);
                    dissassembly.push_str(&format!("${:02X}{:02X} = {:02X}",parameters[1],parameters[0],value));
                }
            },
            am::AbsoluteX => {
                let address: u16 = parameters[0] as u16 + ((parameters[1] as u16) << 8) + self.x as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("${:02X}{:02X},X @ {:04X} = {:02X}",parameters[1],parameters[0],address,value));
            },
            am::AbsoluteY => {
                let address: u16 = ((parameters[0] as u32 + ((parameters[1] as u32) << 8) + self.y as u32) % 0x10000) as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("${:02X}{:02X},Y @ {:04X} = {:02X}",parameters[1],parameters[0],address,value));
            },
            am::Indirect => {
                let ptr: u16 = parameters[0] as u16 + ((parameters[1] as u16) << 8);
                let address_lo: u8;
                let address_hi: u8;
                if (ptr & 0x00FF) == 0x00FF {
                    // Hardware bug
                    address_lo = self.read_only_bus(ptr);
                    address_hi = self.read_only_bus(ptr & 0xFF00);
                }
                else {
                    address_lo = self.read_only_bus(ptr);
                    address_hi = self.read_only_bus(ptr + 1);
                }
                let address: u16 = address_lo as u16 + ((address_hi as u16) << 8);
                dissassembly.push_str(&format!("(${:02X}{:02X}) = {:04X}",parameters[1],parameters[0],address));
            },
            am::IndirectX => {
                let ptr_lo: u16 = (parameters[0] as u16 + self.x as u16) % 0x100;
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address: u16 = self.read_only_bus(ptr_lo) as u16 + ((self.read_only_bus(ptr_hi) as u16) << 8);
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}",parameters[0],ptr_lo,address,value));
            },
            am::IndirectY => {
                let ptr_lo: u16 = (parameters[0] as u16) % 0x100;
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address_ptr: u16 = self.read_only_bus(ptr_lo) as u16 + ((self.read_only_bus(ptr_hi) as u16) << 8);
                let address: u16 = ((address_ptr as u32 + self.y as u32) % 0x10000) as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}",parameters[0],address_ptr,address,value));
            },
            am::NoMode => {
                panic!("No mode specified when trying to fetch data !");
            }
        };
        while dissassembly.len() < 32 {
            dissassembly.push_str(" ");
        }
        dissassembly
    }
}