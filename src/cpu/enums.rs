// Implements the required enums for the CPU emulation

// ===== ENUMS =====

// All the possible adressing modes of the CPU
#[derive(Clone, Copy, PartialEq)]
pub enum AdressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    NoMode,
}

// Flags for the status register
pub enum Flag {
    Carry = 1,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal = 1 << 3,
    Break = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

// Interrupts type
#[derive(Debug, PartialEq)]
pub enum Interrupt {
    Irq,
    Nmi,
    Reset,
}
