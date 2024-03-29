pub mod enums;
pub mod state;

mod instructions;

// Implements the CPU of the NES, i.e. a component
// with a similar behavior as the rp2A03 / 6502

// ====== IMPORTS =====

use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::Bus;
use crate::bus::STACK_OFFSET;
use crate::state::Stateful;
use enums::{AdressingMode as am, Flag, Interrupt};
use instructions::{CpuInstruction, INSTRUCTIONS};

use self::state::CpuState;

// ===== CPU STRUCT =====

// This struct contains the various registers of the CPU
pub struct Cpu {
    // Registers
    a: u8,   // accumulator
    x: u8,   // x index
    y: u8,   // y index
    pc: u16, // program counter
    sp: u8,  // stack pointer
    p: u8,   // status flags

    // Cycles required by the current instruction to complete
    cycles: u8,

    // Does the current instruction require an eventual additional cycle ?
    require_add_cycle: bool,

    // Was a page crossed during addressing ?
    page_crossed: bool,

    // Total clock cycles from the start of the CPU
    total_clock: u64,

    // Display the log of the CPU
    display_logs: bool,

    // pointer to the data bus where we read from and write to
    p_bus: Rc<RefCell<Bus>>,
}

impl Cpu {
    pub fn new(p_bus: Rc<RefCell<Bus>>, display_logs: bool) -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            p: 0x34,

            cycles: 0,

            require_add_cycle: false,
            page_crossed: false,

            total_clock: 0,

            display_logs,

            p_bus,
        }
    }

    pub fn from_state(state: &CpuState, p_bus: Rc<RefCell<Bus>>, display_logs: bool) -> Self {
        let mut cpu = Cpu::new(p_bus, display_logs);
        cpu.set_state(state);
        cpu
    }

    // ===== BUS ACCESS =====

    // Reads data from the bus at the given address
    fn read_bus(&self, address: u16) -> u8 {
        match self.p_bus.borrow_mut().read(address) {
            Ok(data) => data,
            Err(e) => panic!("{}", e),
        }
    }

    // Writes data to the bus at the given address
    fn write_bus(&mut self, address: u16, data: u8) {
        match self.p_bus.borrow_mut().write(address, data) {
            Ok(data) => data,
            Err(e) => panic!("{}", e),
        }
    }

    // Pushes data to stack
    fn push_to_stack(&mut self, data: u8) {
        self.write_bus(STACK_OFFSET + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    // Returns data from the stack
    fn pop_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read_bus(STACK_OFFSET + self.sp as u16)
    }

    // ===== FLAG SETTER AND GETTER =====

    // Returns the required flag from the status register
    fn get_flag(&self, flag: Flag) -> bool {
        self.p & flag as u8 != 0
    }

    // Toggles the required flag in the status register
    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.p |= flag as u8;
        } else {
            self.p &= !(flag as u8);
        }
    }

    // ===== INTERRUPTS =====

    // Called when an interrupt occurs
    pub fn interrupt(&mut self, interrupt_type: Interrupt) {
        match interrupt_type {
            Interrupt::Nmi => self.nmi_interrupt(),
            Interrupt::Irq => self.irq_interrupt(),
            Interrupt::Reset => self.reset_interrupt(),
        }
    }

    fn nmi_interrupt(&mut self) {
        // Push program counter and status register on the stack
        self.push_to_stack(((self.pc & 0xFF00) >> 8) as u8);
        self.push_to_stack((self.pc & 0x00FF) as u8);
        self.push_to_stack(self.p);

        // Disable interrupts
        self.set_flag(Flag::InterruptDisable, true);

        // Load interrupt handler address into the program counter
        let start_address = 0xFFFA;
        self.pc = self.read_bus(start_address) as u16
            + ((self.read_bus(start_address + 1) as u16) << 8) as u16;

        self.cycles = 7;
    }

    fn irq_interrupt(&mut self) {
        if !self.get_flag(Flag::InterruptDisable) {
            // Push program counter and status register on the stack
            self.push_to_stack(((self.pc & 0xFF00) >> 8) as u8);
            self.push_to_stack((self.pc & 0x00FF) as u8);
            self.push_to_stack(self.p);

            // Disable interrupts
            self.set_flag(Flag::InterruptDisable, true);

            // Load interrupt handler address into the program counter
            let start_address = 0xFFFE;
            self.pc = self.read_bus(start_address) as u16
                + ((self.read_bus(start_address + 1) as u16) << 8) as u16;

            self.cycles = 7;
        }
    }

    fn reset_interrupt(&mut self) {
        // Decrease stack pointer by 3 without pushing anything to the stack
        self.sp = self.sp.wrapping_sub(3);

        // Disable interrupts
        self.set_flag(Flag::InterruptDisable, true);

        // Load interrupt handler address into the program counter
        let start_address = 0xFFFC;
        self.pc = self.read_bus(start_address) as u16
            + ((self.read_bus(start_address + 1) as u16) << 8) as u16;

        self.cycles = 7;
    }

    // Called when the reset button is pressed on the NES
    pub fn reset(&mut self) {
        self.interrupt(Interrupt::Reset);
    }

    // ===== CALLED BY NES =====

    // Executes a clock cycle
    pub fn clock(&mut self) {
        // cycle 0 does the operation and the others do nothing
        if self.cycles == 0 {
            // Get operation code
            let opcode: u8 = self.read_bus(self.pc);

            // Logs
            if self.display_logs {
                self.display_cpu_log(opcode);
            }

            // Get instruction information for the operation code
            let instruction: &CpuInstruction = &INSTRUCTIONS[opcode as usize];
            self.require_add_cycle = instruction.add_cycle;

            // Execute the instruction
            (instruction.execute)(self, instruction.adressing_mode);

            // Increase program counter
            self.pc = self.pc.wrapping_add(1);

            // Sets the correct number of cycles
            self.cycles += instruction.cycles - 1;
        } else {
            self.cycles -= 1;
        }
        self.total_clock = self.total_clock.wrapping_add(1);
    }

    // Set the program counter at a specific address
    pub fn set_program_counter_at(&mut self, address: u16) {
        self.pc = address;
    }

    // ===== ADDRESSING MODES =====

    // Returns the parameters for the instruction as an address
    fn fetch_address(&mut self, mode: am) -> u16 {
        self.page_crossed = false;
        match mode {
            am::Implicit => 0,
            am::Accumulator => self.a as u16,
            am::Immediate => {
                self.pc += 1;
                self.pc
            }
            am::ZeroPage => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                address as u16
            }
            am::ZeroPageX => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                (address as u16 + self.x as u16) % 0x100
            }
            am::ZeroPageY => {
                self.pc += 1;
                let address: u8 = self.read_bus(self.pc);
                if (address as u16 + self.y as u16) & 0x0100 > 0 {
                    self.read_bus(
                        (address as u16 & 0xFF00)
                            | (((address as u16).wrapping_add(self.x as u16)) & 0x00FF),
                    ); // Dummy read
                }
                (address as u16 + self.y as u16) % 0x100
            }
            am::Relative => {
                self.pc += 1;
                self.pc
            }
            am::Absolute => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                lo as u16 + ((hi as u16) << 8)
            }
            am::AbsoluteX => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let address: u16 = lo as u16 + ((hi as u16) << 8);
                let result = (address as u32 + self.x as u32) as u16;
                if (result & 0xFF00) != (address & 0xFF00) {
                    if self.require_add_cycle {
                        self.cycles += 1;
                    }
                    self.read_bus((address & 0xFF00) | (result & 0x00FF)); // Dummy read
                    self.page_crossed = true;
                }
                result
            }
            am::AbsoluteY => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let address: u16 = lo as u16 + ((hi as u16) << 8);
                let result = (address as u32 + self.y as u32) as u16;
                if (result & 0xFF00) != (address & 0xFF00) {
                    if self.require_add_cycle {
                        self.cycles += 1;
                    }
                    self.read_bus((address & 0xFF00) | (result & 0x00FF)); // Dummy read
                    self.page_crossed = true;
                }
                result
            }
            am::Indirect => {
                self.pc += 1;
                let lo: u8 = self.read_bus(self.pc);
                self.pc += 1;
                let hi: u8 = self.read_bus(self.pc);
                let ptr: u16 = lo as u16 + ((hi as u16) << 8);
                let (address_lo, address_hi) = if lo == 0xFF {
                    // Hardware bug
                    (self.read_bus(ptr), self.read_bus(ptr & 0xFF00))
                } else {
                    (self.read_bus(ptr), self.read_bus(ptr + 1))
                };
                address_lo as u16 + ((address_hi as u16) << 8)
            }
            am::IndirectX => {
                self.pc += 1;
                let ptr_lo: u16 = (self.read_bus(self.pc) as u16 + self.x as u16) % 0x100; // address in the 0x00 page
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                self.read_bus(ptr_lo) as u16 + ((self.read_bus(ptr_hi) as u16) << 8)
            }
            am::IndirectY => {
                self.pc += 1;
                let ptr_lo: u16 = (self.read_bus(self.pc) as u16) % 0x100; // address in the 0x00 page
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address: u16 =
                    self.read_bus(ptr_lo) as u16 + ((self.read_bus(ptr_hi) as u16) << 8);
                let result: u16 = ((address as u32 + self.y as u32) % 0x1_0000) as u16;
                if (result & 0xFF00) != (address & 0xFF00) {
                    if self.require_add_cycle {
                        self.cycles += 1;
                    }
                    self.read_bus((address & 0xFF00) | (result & 0x00FF)); // Dummy read
                    self.page_crossed = true;
                }
                result
            }
            am::NoMode => {
                panic!("No mode specified when trying to fetch data");
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
        self.set_flag(
            Flag::Overflow,
            (previous_a ^ data) & 0x80 == 0 && (previous_a ^ result as u8) & 0x80 == 0x80,
        );
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
        let result = if mode == am::Accumulator {
            let shifted = (self.a as u16) << 1;
            self.a = shifted as u8;
            shifted
        } else {
            let shifted = data << 1;
            self.write_bus(address, shifted as u8);
            shifted
        };
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
        self.set_flag(Flag::Negative, (data & 0x80) > 0);
        self.set_flag(Flag::Overflow, (data & 0x40) > 0);
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
        self.set_flag(Flag::Unused, true);
        self.interrupt(Interrupt::Irq);
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
        let result = self.a as i16 - data as i16;
        self.set_flag(Flag::Zero, result as u8 == 0x00);
        self.set_flag(Flag::Carry, self.a >= data);
        self.set_flag(Flag::Negative, result as u8 & 0x80 > 0);
    }

    // Compare x register
    // Z,C,N = X-M
    pub fn cpx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result = self.x as i16 - data as i16;
        self.set_flag(Flag::Zero, result as u8 == 0x00);
        self.set_flag(Flag::Carry, self.x >= data);
        self.set_flag(Flag::Negative, result as u8 & 0x80 > 0);
    }

    // Compare y register
    // Z,C,N = Y-M
    pub fn cpy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result = self.y as i16 - data as i16;
        self.set_flag(Flag::Zero, result as u8 == 0x00);
        self.set_flag(Flag::Carry, self.y >= data);
        self.set_flag(Flag::Negative, result as u8 & 0x80 > 0);
    }

    // Decrement memory
    // M,Z,N = M-1
    pub fn dec(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address).wrapping_sub(1);
        self.write_bus(address, data);
        self.set_flag(Flag::Zero, data == 0);
        self.set_flag(Flag::Negative, data & 0x80 > 0);
    }

    // Decrement x register
    // X,Z,N = X-1
    pub fn dex(&mut self, _: am) {
        self.x = self.x.wrapping_sub(1);
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 > 0);
    }

    // Decrement y register
    // Y,Z,N = Y-1
    pub fn dey(&mut self, _: am) {
        self.y = self.y.wrapping_sub(1);
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 > 0);
    }

    // Exclusive or
    // A,Z,N = A^M
    pub fn eor(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a ^= data;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
    }

    // Increment memory
    // M,Z,N = M+1
    pub fn inc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result = data.wrapping_add(1);
        self.write_bus(address, result);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, result & 0x80 > 0);
    }

    // Increment x register
    // X,Z,N = X+1
    pub fn inx(&mut self, _: am) {
        self.x = self.x.wrapping_add(1);
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 > 0);
    }

    // Increment y register
    // Y,Z,N = Y+1
    pub fn iny(&mut self, _: am) {
        self.y = self.y.wrapping_add(1);
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 > 0);
    }

    // Jump
    // pc = addr
    pub fn jmp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.pc = address.wrapping_sub(1);
    }

    // Jump to subroutine
    pub fn jsr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        self.push_to_stack((self.pc >> 8) as u8);
        self.push_to_stack(self.pc as u8);
        self.pc = address.wrapping_sub(1);
    }

    // Load accumulator
    // A,Z,N = M
    pub fn lda(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a = data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
    }

    // Load x register
    // X,Z,N = M
    pub fn ldx(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.x = data;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 > 0);
    }

    // Load y register
    // Y,Z,N = M
    pub fn ldy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.y = data;
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 > 0);
    }

    // Logical shift right
    // A,C,Z,N = A/2 or M,C,Z,N = M/2
    pub fn lsr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let (previous, result) = if mode == am::Accumulator {
            let p = self.a;
            self.a >>= 1;
            (p, self.a)
        } else {
            let p = data as u8;
            let r = ((data >> 1) & 0x00FF) as u8;
            self.write_bus(address, r);
            (p, r)
        };
        self.set_flag(Flag::Carry, (previous & 0x01) > 0);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, result & 0x80 > 0);
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
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
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
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
    }

    // Pull processor status
    // status <= stack
    pub fn plp(&mut self, _: am) {
        let status: u8 = self.pop_from_stack();
        self.set_flag(
            Flag::Carry,
            status & (Flag::Carry as u8) == Flag::Carry as u8,
        );
        self.set_flag(Flag::Zero, status & (Flag::Zero as u8) == Flag::Zero as u8);
        self.set_flag(
            Flag::InterruptDisable,
            status & (Flag::InterruptDisable as u8) == Flag::InterruptDisable as u8,
        );
        self.set_flag(
            Flag::Decimal,
            status & (Flag::Decimal as u8) == Flag::Decimal as u8,
        );
        self.set_flag(Flag::Unused, true);
        self.set_flag(
            Flag::Overflow,
            status & (Flag::Overflow as u8) == Flag::Overflow as u8,
        );
        self.set_flag(
            Flag::Negative,
            status & (Flag::Negative as u8) == Flag::Negative as u8,
        );
    }

    // Rotate left
    pub fn rol(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let (previous, result) = if mode == am::Accumulator {
            let p = self.a;
            self.a = (self.a << 1) + (self.get_flag(Flag::Carry) as u8);
            (p, self.a)
        } else {
            let p = data;
            let r = (data << 1) + (self.get_flag(Flag::Carry) as u8);
            self.write_bus(address, r);
            (p, r)
        };
        self.set_flag(Flag::Carry, (previous & 0x80) > 0);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) > 0);
    }

    // Rotate right
    pub fn ror(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let (previous, result) = if mode == am::Accumulator {
            let p = self.a;
            self.a = (self.a >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
            (p, self.a)
        } else {
            let p = data;
            let r = (data >> 1) + (self.get_flag(Flag::Carry) as u8 * 0x80);
            self.write_bus(address, r);
            (p, r)
        };
        self.set_flag(Flag::Carry, (previous & 0x01) > 0);
        self.set_flag(Flag::Zero, result == 0x00);
        self.set_flag(Flag::Negative, (result & 0x80) > 0);
    }

    // Return from interrupt
    // status <= stack, pc <= stack
    pub fn rti(&mut self, _: am) {
        let status: u8 = self.pop_from_stack();
        self.set_flag(
            Flag::Carry,
            status & (Flag::Carry as u8) == Flag::Carry as u8,
        );
        self.set_flag(Flag::Zero, status & (Flag::Zero as u8) == Flag::Zero as u8);
        self.set_flag(
            Flag::InterruptDisable,
            status & (Flag::InterruptDisable as u8) == Flag::InterruptDisable as u8,
        );
        self.set_flag(
            Flag::Decimal,
            status & (Flag::Decimal as u8) == Flag::Decimal as u8,
        );
        self.set_flag(Flag::Unused, true);
        self.set_flag(
            Flag::Overflow,
            status & (Flag::Overflow as u8) == Flag::Overflow as u8,
        );
        self.set_flag(
            Flag::Negative,
            status & (Flag::Negative as u8) == Flag::Negative as u8,
        );
        let address: u16 = self.pop_from_stack() as u16 + ((self.pop_from_stack() as u16) << 8);
        self.pc = address.wrapping_sub(1);
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
        self.set_flag(Flag::Carry, (result & 0x0100) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
        // !!((A ^ Val) & (A ^ result) & 0x80) Only this, taken from Nintendulator could pass the nestest
        self.set_flag(
            Flag::Overflow,
            !!((previous_a ^ original_data) & (previous_a ^ (result as u8)) & 0x80) == 0x80,
        );
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
        self.set_flag(Flag::Negative, (self.x & 0x80) > 0);
    }

    // Transfer accumulator to y
    // Y = A
    pub fn tay(&mut self, _: am) {
        self.y = self.a;
        self.set_flag(Flag::Zero, self.y == 0x00);
        self.set_flag(Flag::Negative, (self.y & 0x80) > 0);
    }

    // Transfer stack pointer to x
    // X = S
    pub fn tsx(&mut self, _: am) {
        self.x = self.sp;
        self.set_flag(Flag::Zero, self.x == 0x00);
        self.set_flag(Flag::Negative, (self.x & 0x80) > 0);
    }

    // Transfer x to accumulator
    // A = X
    pub fn txa(&mut self, _: am) {
        self.a = self.x;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
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
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
    }

    // ===== UNDOCUMENTED OPCODES =====

    // Same as AND, with C flag
    // A,C,Z,N = A & M
    pub fn anc(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
        self.set_flag(Flag::Carry, self.a & 0x80 > 0);
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
        self.set_flag(Flag::Negative, self.a & 0x80 > 0);
    }

    // Same as AND + ROR
    // C = bit 6
    // V = bit 5 != bit 6
    pub fn arr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a &= data;
        self.a = (self.a >> 1) + ((self.get_flag(Flag::Carry) as u8) << 7);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
        self.set_flag(Flag::Carry, (self.a & 0x40) > 0);
        self.set_flag(
            Flag::Overflow,
            ((self.a & 0x20) > 0) as u8 != ((self.a & 0x40) > 0) as u8,
        ); // bit 5 != bit 6
    }

    // Same as AND + shift right
    pub fn asr(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let tmp: u8 = self.a & data;
        self.a = (tmp >> 1) as u8;
        self.set_flag(Flag::Carry, (tmp & 0x01) > 0);
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
    }

    // Same as DEC + CMP
    // M,C,Z,N = M-1
    pub fn dcp(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address).wrapping_sub(1);
        self.write_bus(address, data);
        let result = self.a as i16 - data as i16;
        self.set_flag(Flag::Zero, result as u8 == 0x00);
        self.set_flag(Flag::Carry, self.a >= data);
        self.set_flag(Flag::Negative, (result as u8 & 0x80) > 0);
    }

    // Same as INC + SBC
    // M = M+1
    // A,Z,C,N,V = A-M-(1-C)
    pub fn isb(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let inc_data: u8 = self.read_bus(address).wrapping_add(1);
        self.write_bus(address, inc_data);
        let data: u8 = inc_data ^ 0xFF; // Converts data into a negative value + 1
        let result: u16 = self.a as u16 + data as u16 + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
        self.set_flag(
            Flag::Overflow,
            !!((previous_a ^ inc_data) & (previous_a ^ (result as u8)) & 0x80) == 0x80,
        );
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
        self.set_flag(Flag::Negative, (tmp & 0x80) > 0);
    }

    // Same as LDA + LDX
    // A,X,N,Z = M
    pub fn lax(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a = data;
        self.x = data;
        self.set_flag(Flag::Zero, data == 0x00);
        self.set_flag(Flag::Negative, (data & 0x80) > 0);
    }

    // Same as ORA #$EE + AND + TXA
    pub fn lxa(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        self.a = data;
        self.x = self.a;
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
    }

    // Same as ROL + AND
    pub fn rla(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let result: u8 = (data << 1) + (self.get_flag(Flag::Carry) as u8);
        self.write_bus(address, result);
        self.a &= result;
        self.set_flag(Flag::Carry, (data & 0x80) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
    }

    // Same as ROR + ADC
    pub fn rra(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u8 = self.read_bus(address);
        let rored = (data >> 1) + ((self.get_flag(Flag::Carry) as u8) << 7);
        self.write_bus(address, rored);
        self.set_flag(Flag::Carry, (data & 0x01) > 0);

        let result: u16 = self.a as u16 + rored as u16 + self.get_flag(Flag::Carry) as u16;
        let previous_a: u8 = self.a;
        self.a = result as u8;
        self.set_flag(Flag::Carry, (result & 0x0100) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
        self.set_flag(
            Flag::Overflow,
            (previous_a ^ rored) & 0x80 == 0 && (previous_a ^ result as u8) & 0x80 == 0x80,
        );
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
        let data: u8 = self.read_bus(address);
        let anded = self.x & self.a;
        let result = anded as i16 - data as i16;
        self.x = result as u8;
        self.set_flag(Flag::Zero, result as u8 == 0x00);
        self.set_flag(Flag::Carry, anded >= data);
        self.set_flag(Flag::Negative, (result as u8 & 0x80) > 0);
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
        let address = if self.page_crossed {
            (address & 0x00FF) + ((result as u16) << 8)
        } else {
            address
        };
        self.write_bus(address, result);
    }

    // M = Y&(h[M]+1)
    pub fn shy(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let result: u8 = self.y & (((address & 0xFF00) >> 8) + 1) as u8;
        let address = if self.page_crossed {
            (address & 0x00FF) + ((result as u16) << 8)
        } else {
            address
        };
        self.write_bus(address, result);
    }

    // Same as ASL + ORA
    pub fn slo(&mut self, mode: am) {
        let address: u16 = self.fetch_address(mode);
        let data: u16 = self.read_bus(address) as u16;
        let result = (data as u16) << 1;
        self.write_bus(address, result as u8);
        self.a |= result as u8;
        self.set_flag(Flag::Carry, (result & 0xFF00) > 0);
        self.set_flag(Flag::Zero, self.a == 0x00);
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
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
        self.set_flag(Flag::Negative, (self.a & 0x80) > 0);
    }

    // Used for unvalid operation codes
    pub fn err(&mut self, _: am) {
        panic!("Encountered an unvalid opcode at {:#X}", self.pc);
    }

    // ===== DEBUGGING =====

    fn display_cpu_log(&self, opcode: u8) {
        let mut instruction_and_parameters_str = format!("{:02X} ", opcode);
        let mut instruction_parameters: Vec<u8> = vec![];
        for i in 0..INSTRUCTIONS[opcode as usize].bytes - 1 {
            instruction_parameters.push(self.read_only_bus(self.pc + i as u16 + 1));
            instruction_and_parameters_str
                .push_str(&format!("{:02X} ", instruction_parameters[i as usize]));
        }
        while instruction_and_parameters_str.len() < 9 {
            instruction_and_parameters_str.push(' ');
        }
        let cpu_log: String = format!(
            "{:04X}  {} {}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.pc,
            instruction_and_parameters_str,
            self.dissassemble(opcode, instruction_parameters),
            self.a,
            self.x,
            self.y,
            self.p,
            self.sp
        );

        let mut scanline_str = self.p_bus.borrow().get_scanline().to_string();
        while scanline_str.len() < 3 {
            scanline_str = format!(" {}", scanline_str);
        }
        let mut cycle_str = self.p_bus.borrow().get_cycles().to_string();
        while cycle_str.len() < 3 {
            cycle_str = format!(" {}", cycle_str);
        }
        let ppu_log = format!("PPU:{},{}", scanline_str, cycle_str);

        println!("{} {} CYC:{}", cpu_log, ppu_log, self.total_clock);
    }

    fn read_only_bus(&self, address: u16) -> u8 {
        match self.p_bus.borrow().read_only(address) {
            Ok(data) => data,
            Err(e) => panic!("{}", e),
        }
    }

    fn dissassemble(&self, opcode: u8, parameters: Vec<u8>) -> String {
        let mut dissassembly = String::from(INSTRUCTIONS[opcode as usize].name);
        dissassembly.push(' ');
        match INSTRUCTIONS[opcode as usize].adressing_mode {
            am::Accumulator => {
                if (opcode != 0xAA) && (opcode != 0x8A) {
                    dissassembly.push('A');
                }
            }
            am::Implicit => (),
            am::Immediate => dissassembly.push_str(&format!("#${:02X}", parameters[0])),
            am::ZeroPage => {
                let value: u8 = self.read_only_bus(parameters[0] as u16);
                dissassembly.push_str(&format!("${:02X} = {:02X}", parameters[0], value));
            }
            am::ZeroPageX => {
                let address: u16 = (parameters[0] as u16 + self.x as u16) % 0x100;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "${:02X},X @ {:02X} = {:02X}",
                    parameters[0], address, value
                ));
            }
            am::ZeroPageY => {
                let address: u16 = (parameters[0] as u16 + self.y as u16) % 0x100;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "${:02X},Y @ {:02X} = {:02X}",
                    parameters[0], address, value
                ));
            }
            am::Relative => {
                dissassembly.push_str(&format!(
                    "${:04X}",
                    (self.pc as i16) + 2 + ((parameters[0] as i8) as i16)
                ));
            }
            am::Absolute => {
                let address: u16 = parameters[0] as u16 + ((parameters[1] as u16) << 8) as u16;
                // Don't print value if it's a JMP / JSR
                if (opcode == 0x4C) || (opcode == 0x20) {
                    dissassembly.push_str(&format!("${:02X}{:02X}", parameters[1], parameters[0]));
                } else {
                    let value: u8 = self.read_only_bus(address);
                    dissassembly.push_str(&format!(
                        "${:02X}{:02X} = {:02X}",
                        parameters[1], parameters[0], value
                    ));
                }
            }
            am::AbsoluteX => {
                let address: u16 =
                    (parameters[0] as u32 + ((parameters[1] as u32) << 8) + self.x as u32) as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "${:02X}{:02X},X @ {:04X} = {:02X}",
                    parameters[1], parameters[0], address, value
                ));
            }
            am::AbsoluteY => {
                let address: u16 =
                    ((parameters[0] as u32 + ((parameters[1] as u32) << 8) + self.y as u32)
                        % 0x10000) as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "${:02X}{:02X},Y @ {:04X} = {:02X}",
                    parameters[1], parameters[0], address, value
                ));
            }
            am::Indirect => {
                let ptr: u16 = parameters[0] as u16 + ((parameters[1] as u16) << 8);
                let (address_lo, address_hi) = if (ptr & 0x00FF) == 0x00FF {
                    // Hardware bug
                    (self.read_only_bus(ptr), self.read_only_bus(ptr & 0xFF00))
                } else {
                    (self.read_only_bus(ptr), self.read_only_bus(ptr + 1))
                };
                let address: u16 = address_lo as u16 + ((address_hi as u16) << 8);
                dissassembly.push_str(&format!(
                    "(${:02X}{:02X}) = {:04X}",
                    parameters[1], parameters[0], address
                ));
            }
            am::IndirectX => {
                let ptr_lo: u16 = (parameters[0] as u16 + self.x as u16) % 0x100;
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address: u16 =
                    self.read_only_bus(ptr_lo) as u16 + ((self.read_only_bus(ptr_hi) as u16) << 8);
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    parameters[0], ptr_lo, address, value
                ));
            }
            am::IndirectY => {
                let ptr_lo: u16 = (parameters[0] as u16) % 0x100;
                let ptr_hi: u16 = (ptr_lo + 1) % 0x100;
                let address_ptr: u16 =
                    self.read_only_bus(ptr_lo) as u16 + ((self.read_only_bus(ptr_hi) as u16) << 8);
                let address: u16 = ((address_ptr as u32 + self.y as u32) % 0x10000) as u16;
                let value: u8 = self.read_only_bus(address);
                dissassembly.push_str(&format!(
                    "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    parameters[0], address_ptr, address, value
                ));
            }
            am::NoMode => {
                panic!("No mode specified when trying to fetch data !");
            }
        };
        while dissassembly.len() < 32 {
            dissassembly.push(' ');
        }
        dissassembly
    }
}
