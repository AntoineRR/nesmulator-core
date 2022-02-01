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

    pub fn black() -> Self {
        ARGBColor::new(0, 0, 0, 0)
    }

    pub fn light_gray() -> Self {
        ARGBColor::new(255, 50, 50, 50)
    }
}