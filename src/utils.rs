use serde::{Deserialize, Serialize};

/// A utiliy struct to represent an color.
/// The alpha channel is not calculated by the emulator (set to 255).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ARGBColor {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ARGBColor {
    /// Create a new color
    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        ARGBColor {
            alpha,
            red,
            green,
            blue,
        }
    }

    /// Return a black color (255, 0, 0, 0)
    pub fn black() -> Self {
        ARGBColor::new(255, 0, 0, 0)
    }

    /// Return a light gray color (255, 50, 50, 50)
    pub fn light_gray() -> Self {
        ARGBColor::new(255, 50, 50, 50)
    }
}

/// A utility enum to represent each input possible on a NES controller.
#[derive(Debug)]
pub enum ControllerInput {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Down = 0b0000_0100,
    Up = 0b0000_1000,
    Start = 0b0001_0000,
    Select = 0b0010_0000,
    B = 0b0100_0000,
    A = 0b1000_0000,
}
