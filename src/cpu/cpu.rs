// Implements the CPU of the NES, i.e. a component
// with a similar behavior as the rp2A03 / 6502

// ====== IMPORTS =====

use std::sync::{Arc, Mutex};

use crate::{bus::Bus};
use crate::bus::STACK_OFFSET;
use super::cpu_instructions::{CpuInstruction,INSTRUCTIONS};
use super::cpu_enums::{AdressingMode as am,Flag,Interrupt};

// ===== CONSTANTS =====

//pub const CPU_FREQUENCY: u32 = 1789773; // NTSC NES / Famicom frequency (Hz)

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
        if !self.get_flag(Flag::InterruptDisable) {
            // Push program counter and status register on the stack
            self.write_bus(STACK_OFFSET + self.sp as u16, ((self.pc & 0xFF00) >> 8) as u8);
            self.sp -= 1;
            self.write_bus(STACK_OFFSET + self.sp as u16, (self.pc & 0x00FF) as u8);
            self.sp -= 1;
            self.write_bus(STACK_OFFSET + self.sp as u16, self.p);
            self.sp -= 1;
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

            self.cycles = 7;
        }
    }

    // Called when the reset button is pressed on the NES
    pub fn reset(&mut self) {
        println!("RESET");
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.sp = 0xFD;
        self.p = 0x00;
        self.interrupt(Interrupt::Reset);
    }

    // ===== CLOCK =====

    // Executes a clock cycle
    // opcodes for operations are stored in the INSTRUCTIONS const
    pub fn clock(&mut self) {
        if self.cycles == 0 {
            let opcode: u8 = self.read_bus(self.pc);
            //println!("opcode : {:#x}", opcode);
            let instruction: &CpuInstruction = &INSTRUCTIONS[opcode as usize];
            (instruction.execute)(self, instruction.adressing_mode);
            self.pc += 1;
            self.cycles += instruction.cycles - 1;
        }
        else {
            self.cycles -= 1;
        }
    }

    // ===== ADDRESSING MODES =====

    // Returns the parameters for the instruction as an address
    pub fn fetch_address(&mut self, mode: am) -> u16 {
        match mode {
            am::Accumulator => {
                self.pc += 1;
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
                address as u16 + self.x as u16
            },
            am::ZeroPageY => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                address as u16 + self.y as u16
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
                if (result & 0xFF00) != (address & 0xFF00) {
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
                let result: u16 = address + self.y as u16;
                if (result & 0xFF00) != (address & 0xFF00) {
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
                let address_lo: u8 = self.read_bus(ptr);
                let address_hi: u8 = self.read_bus(ptr + 1);
                address_lo as u16 + (address_hi as u16) << 8
            },
            am::IndirectX => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let ptr: u16 = lo as u16 + ((hi as u16) << 8);
                let address_lo: u8 = self.read_bus(ptr);
                let address_hi: u8 = self.read_bus(ptr + 1);
                address_lo as u16 + (address_hi as u16) << 8 + self.x as u16
            },
            am::IndirectY => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let ptr: u16 = lo as u16 + ((hi as u16) << 8);
                let address_lo: u8 = self.read_bus(ptr);
                let address_hi: u8 = self.read_bus(ptr + 1);
                let address: u16 = address_lo as u16 + (address_hi as u16) << 8;
                let result: u16 = address + self.y as u16;
                if (result & 0xFF00) != (address & 0xFF00) {
                    self.cycles += 1;
                }
                result
            },
            am::NoMode => {
                panic!("No mode specified when trying to fetch data !");
            }
            _ => panic!("Invalid mode used !")
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
        self.set_flag(Flag::Overflow, !(previous_a ^ data) & (previous_a ^ self.a) == 0x01);
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
        let result: u8 = ((data << 1) & 0x00FF) as u8;
        match mode {
            am::Accumulator => self.a = result,
            _ => self.write_bus(address, result)
        }
        self.set_flag(Flag::Carry, ((data << 1) & 0xFF00) > 0);
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, result & 0x80 == 0x80);
    }

    // Branch if carry clear
    // (C = 0) => pc += addr
    pub fn bcc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Carry) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
        self.set_flag(Flag::Negative, (result & 0b1000_0000) == 0b1000_0000);
        self.set_flag(Flag::Overflow, (result & 0b0100_0000) == 0b0100_0000);
    }

    // Branch if minus
    // (N = 1) => pc += addr
    pub fn bmi(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if self.get_flag(Flag::Negative) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles += 1;
            }
            self.pc = result as u16;
        }
    }

    // Force interrupt
    pub fn brk(&mut self, _: am) {
        self.interrupt(Interrupt::IRQ);
        self.set_flag(Flag::Break, true);
    }

    // Branch if overflow clear
    // (V = 0) => pc += addr
    pub fn bvc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        if !self.get_flag(Flag::Overflow) {
            let data: i8 = self.read_bus(address) as i8;
            let result: i16 = self.pc as i16 + data as i16;
            self.cycles += 1;
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
            if (result as u16 & 0xFF00) != (self.pc & 0xFF00) {
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
        let result: u8 = self.a - data;
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, result > 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Compare x register
    // Z,C,N = X-M
    pub fn cpx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = self.x - data;
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, result > 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Compare y register
    // Z,C,N = Y-M
    pub fn cpy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = self.y - data;
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Carry, result > 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Decrement memory
    // M,Z,N = M-1
    pub fn dec(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = data - 1;
        self.write_bus(address, result);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Decrement x register
    // X,Z,N = X-1
    pub fn dex(&mut self, _: am) {
        self.x -= 1;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Decrement y register
    // Y,Z,N = Y-1
    pub fn dey(&mut self, _: am) {
        self.y -= 1;
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
        let result: u8 = data + 1;
        self.write_bus(address, result);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Increment x register
    // X,Z,N = X+1
    pub fn inx(&mut self, _: am) {
        self.x += 1;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) == 0x80);
    }

    // Increment y register
    // Y,Z,N = Y+1
    pub fn iny(&mut self, _: am) {
        self.y += 1;
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) == 0x80);
    }

    // Jump
    // pc = addr
    pub fn jmp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.pc = address;
    }

    // Jump to subroutine
    pub fn jsr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.write_bus(STACK_OFFSET + self.sp as u16, (self.pc >> 8) as u8);
        self.sp -= 1;
        self.write_bus(STACK_OFFSET + self.sp as u16, self.pc as u8 + 0x01);
        self.sp -= 1;
        self.pc = address;
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
        let result: u8 = ((data >> 1) & 0x00FF) as u8;
        match mode {
            am::Accumulator => self.a = result,
            _ => self.write_bus(address, result)
        }
        self.set_flag(Flag::Carry, (data & 0x01) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, result & 0x80 == 0x80);
    }

    // No operation
    pub fn nop(&mut self, _: am) {
        
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
        let address: u16 = STACK_OFFSET + self.sp as u16;
        self.write_bus(address, self.a);
        self.sp -= 1;
    }

    // Push processor status
    // status => stack
    pub fn php(&mut self, _: am) {
        let address: u16 = STACK_OFFSET + self.sp as u16;
        self.write_bus(address, self.p);
        self.sp -= 1;
    }

    // Pull accumulator
    // A <= stack
    pub fn pla(&mut self, _: am) {
        self.sp += 1;
        let address: u16 = STACK_OFFSET + self.sp as u16;
        self.a = self.read_bus(address);
    }

    // Pull processor status
    // status <= stack
    pub fn plp(&mut self, _: am) {
        self.sp += 1;
        let address: u16 = STACK_OFFSET + self.sp as u16;
        self.p = self.read_bus(address);
    }

    // Rotate left
    pub fn rol(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = (data << 1) + self.get_flag(Flag::Carry) as u8;
        match mode {
            am::Accumulator => self.a = result,
            _ => self.write_bus(address, result)
        }
        self.set_flag(Flag::Carry, (data & 0x80) == 0x80);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Rotate right
    pub fn ror(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = (data >> 1) + (self.get_flag(Flag::Carry) as u8 * 0b1000_0000);
        match mode {
            am::Accumulator => self.a = result,
            _ => self.write_bus(address, result)
        }
        self.set_flag(Flag::Carry, (data & 0x01) == 0x01);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) == 0x80);
    }

    // Return from interrupt
    // status <= stack, pc <= stack
    pub fn rti(&mut self, _: am) {
        self.sp += 1;
        self.p = self.read_bus(STACK_OFFSET + self.sp as u16);
        self.sp += 1;
        let mut address: u16 = (self.read_bus(STACK_OFFSET + self.sp as u16) as u16) << 8;
        self.sp += 1;
        address += self.read_bus(STACK_OFFSET + self.sp as u16) as u16;
        self.pc = address;
    }

    // Return from subroutine
    // pc - 1 <= stack
    pub fn rts(&mut self, _: am) {
        self.sp += 1;
        let mut address: u16 = (self.read_bus(STACK_OFFSET + self.sp as u16) as u16) << 8;
        self.sp += 1;
        address += self.read_bus(STACK_OFFSET + self.sp as u16) as u16;
        self.pc = address + 1;
    }

    // Substract with carry
    // A,Z,C,N = A-M-(1-C)
    pub fn sbc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let original_data: u8 = self.read_bus(address);
        let data: u16 = (original_data as u16) ^ 0x00FF; // Converts data into a negative value
        let result: u16 = self.a as u16 + data + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) == 0x0100);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) == 0x80);
        self.set_flag(Flag::Overflow, !(previous_a ^ original_data) & (previous_a ^ self.a) == 0x01);
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

    // Used for unvalid operation codes
    pub fn err(&mut self, _: am) {
        panic!("Encountered an unvalid opcode !");
    }
}