// Implements the required enum for PPU emulation

pub enum StatusFlag {
    VBlank = 1 << 7,
    Sprite0Hit = 1 << 6,
    SpriteOverflow = 1 << 5
}

#[derive(PartialEq, Clone, Copy)]
pub enum ControlFlag {
    VBlank = 1 << 7,                        // Should we generate a NMI at the start of VBlank
    MasterSlaveSelect = 1 << 6,
    SpriteSize = 1 << 5,                    // 0 => 8*8, 1 => 8*16
    BackgroundPatternTableAddress = 1 << 4, // 0 => 0x0000, 1 => 0x1000
    SpritePatternTableAddress = 1 << 3,     // 0 => 0x0000, 1 => 0x1000
    VRAMAddressIncrement = 1 << 2,          // When PPU Data is read, 0 => 1 (horizontal), 1 => 32 (vertical)
    NametableAddress = 1                    // 0 => 2000, 1 => 2400, 2 => 2800, 3 => 2C00
}

#[derive(Clone, Copy)]
pub enum MaskFlag {
    EmphasizeBlue = 1 << 7,
    EmphasizeGreen = 1 << 6,
    EmphasizeRed = 1 << 5,
    ShowSprites = 1 << 4,
    ShowBackground = 1 << 3,
    ShowOffScreenSprites = 1 << 2,
    ShowOffScreenBackground = 1 << 1,
    GreyScale = 1
}

pub enum VRAMAddressMask {
    CoarseXScroll = 0x001F,   // 5 lower bits
    CoarseYScroll = 0x03E0,   // 5 next bits
    NametableSelect = 0x0C00, // 2 next bits
    NametableX = 0x0400,      // X bit of the nametable select
    NametableY = 0x0800,      // Y bit of the nametable select
    FineY = 0x7000,           // 3 next bits
    FW2006 = 0x3F00,          // bits set on the first write to 0x2006
    SW2006 = 0x00FF           // bits set on the second write to 0x2006
}