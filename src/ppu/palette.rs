// Implpements the different color palettes of the NES

#[derive(Debug, Clone, Copy)]
pub struct ARGBColor {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ARGBColor {
    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        ARGBColor {
            alpha,
            red,
            green,
            blue,
        }
    }
}

pub const PALETTE: [ARGBColor; 64] = [
    ARGBColor::new(255, 84, 84, 84),    // 0x00
    ARGBColor::new(255, 0, 30, 116),    // 0x01
    ARGBColor::new(255, 8, 16, 144),    // 0x02
    ARGBColor::new(255, 48, 0, 136),    // 0x03
    ARGBColor::new(255, 68, 0, 100),    // 0x04
    ARGBColor::new(255, 92, 0, 48),     // 0x05
    ARGBColor::new(255, 84, 4, 0),      // 0x06
    ARGBColor::new(255, 60, 24, 0),     // 0x07
    ARGBColor::new(255, 32, 42, 0),     // 0x08
    ARGBColor::new(255, 8, 58, 0),      // 0x09
    ARGBColor::new(255, 0, 64, 0),      // 0x0A
    ARGBColor::new(255, 0, 60, 0),      // 0x0B
    ARGBColor::new(255, 0, 50, 60),     // 0x0C
    ARGBColor::new(255, 0, 0, 0),       // 0x0D
    ARGBColor::new(255, 0, 0, 0),       // 0x0E
    ARGBColor::new(255, 0, 0, 0),       // 0x0F
    ARGBColor::new(255, 152, 150, 152), // 0x10
    ARGBColor::new(255, 8, 76, 196),    // 0x11
    ARGBColor::new(255, 48, 50, 236),   // 0x12
    ARGBColor::new(255, 92, 30, 228),   // 0x13
    ARGBColor::new(255, 136, 20, 176),  // 0x14
    ARGBColor::new(255, 160, 20, 100),  // 0x15
    ARGBColor::new(255, 152, 34, 32),   // 0x16
    ARGBColor::new(255, 120, 60, 0),    // 0x17
    ARGBColor::new(255, 84, 90, 0),     // 0x18
    ARGBColor::new(255, 40, 114, 0),    // 0x19
    ARGBColor::new(255, 8, 124, 0),     // 0x1A
    ARGBColor::new(255, 0, 118, 140),   // 0x1B
    ARGBColor::new(255, 0, 102, 120),   // 0x1C
    ARGBColor::new(255, 0, 0, 0),       // 0x1D
    ARGBColor::new(255, 0, 0, 0),       // 0x1E
    ARGBColor::new(255, 0, 0, 0),       // 0x1F
    ARGBColor::new(255, 236, 238, 236), // 0x20
    ARGBColor::new(255, 76, 154, 236),  // 0x21
    ARGBColor::new(255, 120, 124, 236), // 0x22
    ARGBColor::new(255, 176, 98, 236),  // 0x23
    ARGBColor::new(255, 228, 84, 236),  // 0x24
    ARGBColor::new(255, 236, 88, 180),  // 0x25
    ARGBColor::new(255, 236, 106, 100), // 0x26
    ARGBColor::new(255, 212, 136, 32),  // 0x27
    ARGBColor::new(255, 160, 170, 0),   // 0x28
    ARGBColor::new(255, 116, 196, 0),   // 0x29
    ARGBColor::new(255, 76, 208, 32),   // 0x2A
    ARGBColor::new(255, 56, 204, 108),  // 0x2B
    ARGBColor::new(255, 56, 180, 204),  // 0x2C
    ARGBColor::new(255, 60, 60, 60),    // 0x2D
    ARGBColor::new(255, 0, 0, 0),       // 0x2E
    ARGBColor::new(255, 0, 0, 0),       // 0x2F
    ARGBColor::new(255, 236, 238, 236), // 0x30
    ARGBColor::new(255, 168, 204, 236), // 0x31
    ARGBColor::new(255, 188, 188, 236), // 0x32
    ARGBColor::new(255, 212, 178, 236), // 0x33
    ARGBColor::new(255, 236, 174, 236), // 0x34
    ARGBColor::new(255, 236, 174, 212), // 0x35
    ARGBColor::new(255, 236, 180, 176), // 0x36
    ARGBColor::new(255, 228, 196, 144), // 0x37
    ARGBColor::new(255, 204, 210, 120), // 0x38
    ARGBColor::new(255, 180, 222, 120), // 0x39
    ARGBColor::new(255, 168, 226, 144), // 0x3A
    ARGBColor::new(255, 152, 226, 180), // 0x3B
    ARGBColor::new(255, 160, 214, 228), // 0x3C
    ARGBColor::new(255, 160, 162, 160), // 0x3D
    ARGBColor::new(255, 0, 0, 0),       // 0x3E
    ARGBColor::new(255, 0, 0, 0),       // 0x3F
];
